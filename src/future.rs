//! Future instrumentation utilities for async operation monitoring.
//!
//! This module provides wrapper types and traits for instrumenting async operations
//! with callbacks that execute when futures complete, enabling monitoring and
//! metrics collection for async workloads.

use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context as TaskContext, Poll, ready},
};

/// Trait for handling the completion of instrumented futures.
///
/// This trait provides a callback mechanism to perform actions when an instrumented
/// future completes with a result. It's typically used for recording metrics,
/// logging outcomes, or other side effects based on the future's result.
pub trait InstrumentedFutureContext<T> {
    /// Called when the instrumented future completes with a result.
    ///
    /// # Arguments
    ///
    /// - `result`: Reference to the result produced by the future
    fn on_result(self, result: &T);
}

pin_project! {
    /// A future wrapper that provides instrumentation hooks for result handling.
    ///
    /// This future wrapper allows for instrumentation of async operations by providing
    /// a context that is called when the future completes. It ensures that the context
    /// callback is invoked exactly once when the future produces its result.
    ///
    /// # State Management
    ///
    /// The future maintains two states:
    /// - `Pending`: The wrapped future is still executing and contains the future and context
    /// - `Complete`: The future has completed and the context has been invoked
    ///
    /// # Generic Parameters
    ///
    /// - `F`: The wrapped future type
    /// - `C`: The context type that implements [`InstrumentedFutureContext`]
    ///
    /// # Fields
    ///
    /// The `Pending` variant contains the future being instrumented and the context
    /// that will be called when it completes. Field documentation is not possible
    /// within pin_project macros.
    #[project = InstrumentedFutureProj]
    #[project_replace = InstrumentedFutureOwn]
    #[allow(missing_docs)]
    pub enum InstrumentedFuture<F, C>
    where
        F: Future,
        C: InstrumentedFutureContext<F::Output>,
    {
        /// Future is currently executing and waiting for completion
        Pending {
            #[pin]
            future: F,
            context: C,
        },
        /// Future has completed and context has been invoked
        Complete,
    }
}

impl<F, C> InstrumentedFuture<F, C>
where
    F: Future,
    C: InstrumentedFutureContext<F::Output>,
{
    /// Creates a new instrumented future with the given future and context.
    ///
    /// # Arguments
    ///
    /// - `future`: The future to instrument
    /// - `context`: The context that will be called when the future completes
    ///
    /// # Returns
    ///
    /// A new [`InstrumentedFuture`] in the `Pending` state
    ///
    /// # Examples
    ///
    /// ```rust
    /// use telemetry_rust::future::{InstrumentedFuture, InstrumentedFutureContext};
    ///
    /// struct MyContext;
    /// impl InstrumentedFutureContext<i32> for MyContext {
    ///     fn on_result(self, result: &i32) {
    ///         println!("Future completed with result: {}", result);
    ///     }
    /// }
    ///
    /// let future = async { 42 };
    /// let instrumented = InstrumentedFuture::new(future, MyContext);
    /// ```
    pub fn new(future: F, context: C) -> Self {
        Self::Pending { future, context }
    }
}

impl<F, C> Future for InstrumentedFuture<F, C>
where
    F: Future,
    C: InstrumentedFutureContext<F::Output>,
{
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        // First, try to get the ready value of the future
        let ready = match self.as_mut().project() {
            InstrumentedFutureProj::Pending { future, context: _ } => {
                ready!(future.poll(cx))
            }
            InstrumentedFutureProj::Complete => panic!("future polled after completion"),
        };

        // If we got the ready value, we first drop the future: this ensures that the
        // OpenTelemetry span attached to it is closed and included in the subsequent flush.
        let context = match self.project_replace(InstrumentedFuture::Complete) {
            InstrumentedFutureOwn::Pending { future: _, context } => context,
            InstrumentedFutureOwn::Complete => unreachable!("future already completed"),
        };

        context.on_result(&ready);
        Poll::Ready(ready)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert2::assert;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestContext<'a>(&'a AtomicUsize, usize, i32);

    impl InstrumentedFutureContext<i32> for TestContext<'_> {
        fn on_result(self, result: &i32) {
            let Self(counter, expected_count, expected_result) = self;
            assert!(counter.fetch_add(1, Ordering::AcqRel) == expected_count);
            assert!(result == &expected_result);
        }
    }

    #[tokio::test]
    async fn test_hooked_future() {
        let hook_called = AtomicUsize::new(0);
        let fut1 = async { 42 };
        let fut2 = InstrumentedFuture::new(fut1, TestContext(&hook_called, 0, 42));
        let fut3 = InstrumentedFuture::new(fut2, TestContext(&hook_called, 1, 42));

        assert!(hook_called.load(Ordering::Acquire) == 0);
        let res = fut3.await;

        assert!(hook_called.load(Ordering::Acquire) == 2);
        assert!(res == 42);
    }
}
