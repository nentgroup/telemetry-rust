use crate::{future::HookedFuture, semconv};
use lambda_runtime::LambdaInvocation;
use opentelemetry_sdk::trace::TracerProvider;
use std::task::{Context as TaskContext, Poll};
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
    type Future = HookedFuture<Instrumented<S::Future>, TracerProvider>;

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
        HookedFuture::new(future, self.provider.clone(), |provider, _| {
            provider.force_flush();
        })
    }
}
