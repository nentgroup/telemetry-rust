use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{ready, Context as TaskContext, Poll},
};

pub trait HookedFutureContext<T> {
    fn on_result(self, result: &T);
}

pin_project! {
    #[project = HookedFutureProj]
    #[project_replace = HookedFutureReplace]
    pub enum HookedFuture<F: Future, C: HookedFutureContext<F::Output>> {
        Pending {
            #[pin]
            future: F,
            context: C,
        },
        Complete,
    }
}

impl<F: Future, C: HookedFutureContext<F::Output>> HookedFuture<F, C> {
    pub fn new(future: F, context: C) -> Self {
        Self::Pending { future, context }
    }
}

impl<F: Future, C: HookedFutureContext<F::Output>> Future for HookedFuture<F, C> {
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        // First, try to get the ready value of the future
        let ready = match self.as_mut().project() {
            HookedFutureProj::Pending { future, context: _ } => ready!(future.poll(cx)),
            HookedFutureProj::Complete => panic!("future polled after completion"),
        };

        // If we got the ready value, we first drop the future: this ensures that the
        // OpenTelemetry span attached to it is closed and included in the subsequent flush.
        let context = match self.project_replace(HookedFuture::Complete) {
            HookedFutureReplace::Pending { future: _, context } => context,
            HookedFutureReplace::Complete => panic!("future already completed"),
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

    impl HookedFutureContext<i32> for TestContext<'_> {
        fn on_result(self, result: &i32) {
            assert!(self.0.fetch_add(1, Ordering::Relaxed) == self.1);
            assert!(result == &self.2);
        }
    }

    #[tokio::test]
    async fn test_hooked_future() {
        let hook_called = AtomicUsize::new(0);
        let fut1 = async { 42 };
        let fut2 = HookedFuture::new(fut1, TestContext(&hook_called, 0, 42));
        let fut3 = HookedFuture::new(fut2, TestContext(&hook_called, 1, 42));

        let res = fut3.await;

        assert!(res == 42);
    }
}
