use crate::semconv;
use lambda_runtime::LambdaInvocation;
use opentelemetry_sdk::trace::TracerProvider;
use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{ready, Context as TaskContext, Poll};
use tower::{Layer, Service};
use tracing::{instrument::Instrumented, Instrument};

pub struct OtelLambdaLayer {
    provider: TracerProvider,
}

impl OtelLambdaLayer {
    pub fn new(provider: TracerProvider) -> Self {
        Self { provider }
    }
}

impl<S> Layer<S> for OtelLambdaLayer {
    type Service = OtelLambdaService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        OtelLambdaService {
            inner,
            provider: self.provider.clone(),
            coldstart: true,
        }
    }
}

pub struct OtelLambdaService<S> {
    inner: S,
    provider: TracerProvider,
    coldstart: bool,
}

impl<S> Service<LambdaInvocation> for OtelLambdaService<S>
where
    S: Service<LambdaInvocation, Response = ()>,
{
    type Response = ();
    type Error = S::Error;
    type Future = OtelLambdaFuture<Instrumented<S::Future>>;

    fn poll_ready(&mut self, cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: LambdaInvocation) -> Self::Future {
        let span = tracing::info_span!(
            "Lambda function invocation",
            "otel.name" = req.context.env_config.function_name,
            { semconv::FAAS_TRIGGER } = "http",
            { semconv::FAAS_INVOCATION_ID } = req.context.request_id,
            { semconv::FAAS_COLDSTART } = self.coldstart
        );

        self.coldstart = false;

        let future = self.inner.call(req).instrument(span);
        OtelLambdaFuture {
            future: Some(future),
            provider: self.provider.clone(),
        }
    }
}

pin_project! {
    pub struct OtelLambdaFuture<F> {
        #[pin]
        future: Option<F>,
        provider: TracerProvider,
    }
}

impl<F: Future> Future for OtelLambdaFuture<F> {
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        // First, try to get the ready value of the future
        let ready = ready!(self
            .as_mut()
            .project()
            .future
            .as_pin_mut()
            .expect("future polled after completion")
            .poll(cx));

        // If we got the ready value, we first drop the future: this ensures that the
        // OpenTelemetry span attached to it is closed and included in the subsequent flush.
        Pin::set(&mut self.as_mut().project().future, None);

        self.provider.force_flush();
        Poll::Ready(ready)
    }
}
