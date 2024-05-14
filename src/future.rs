use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{ready, Context as TaskContext, Poll},
};

pub trait InstrumentedFutureContext<T> {
    fn on_result(self, result: &T);
}

pin_project! {
    #[project = InstrumentedFutureProj]
    #[project_replace = InstrumentedFutureReplace]
    pub enum InstrumentedFuture<F, C>
    where
        F: Future,
        C: InstrumentedFutureContext<F::Output>,
    {
        Pending {
            #[pin]
            future: F,
            context: C,
        },
        Complete,
    }
}

impl<F, C> InstrumentedFuture<F, C>
where
    F: Future,
    C: InstrumentedFutureContext<F::Output>,
{
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
            InstrumentedFutureReplace::Pending { future: _, context } => context,
            InstrumentedFutureReplace::Complete => panic!("future already completed"),
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
