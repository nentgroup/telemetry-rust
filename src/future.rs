use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{ready, Context as TaskContext, Poll},
};

pin_project! {
    #[project = HookedFutureProj]
    #[project_replace = HookedFutureReplace]
    pub enum HookedFuture<F: Future, C> {
        Pending {
            #[pin]
            future: F,
            context: C,
            ready_hook: fn(C, &F::Output),
        },
        Complete,
    }
}

impl<F: Future, C> HookedFuture<F, C> {
    pub fn new(future: F, context: C, ready_hook: fn(C, &F::Output)) -> Self {
        Self::Pending {
            future,
            context,
            ready_hook,
        }
    }
}

impl<F: Future, C> Future for HookedFuture<F, C> {
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        // First, try to get the ready value of the future
        let ready = match self.as_mut().project() {
            HookedFutureProj::Pending {
                future,
                context: _,
                ready_hook: _,
            } => {
                ready!(future.poll(cx))
            }
            HookedFutureProj::Complete => panic!("future polled after completion"),
        };

        // If we got the ready value, we first drop the future: this ensures that the
        // OpenTelemetry span attached to it is closed and included in the subsequent flush.
        let (context, ready_hook) = match self.project_replace(HookedFuture::Complete) {
            HookedFutureReplace::Pending {
                future: _,
                context,
                ready_hook,
            } => (context, ready_hook),
            HookedFutureReplace::Complete => panic!("future already completed"),
        };

        ready_hook(context, &ready);
        Poll::Ready(ready)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert2::assert;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    async fn func() -> i32 {
        42
    }

    #[tokio::test]
    async fn test_hooked_future() {
        let hook_called = Arc::new(AtomicUsize::new(0));
        let fut1 = func();
        let fut2 = HookedFuture::new(fut1, hook_called.clone(), |hook_called, res| {
            assert!(hook_called.fetch_add(1, Ordering::Relaxed) == 0);
            assert!(res == &42);
        });
        let fut3 = HookedFuture::new(fut2, hook_called.clone(), |hook_called, res| {
            assert!(hook_called.fetch_add(1, Ordering::Relaxed) == 1);
            assert!(res == &42);
        });

        let res = fut3.await;

        assert!(res == 42);
    }
}
