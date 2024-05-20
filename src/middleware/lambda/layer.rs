use crate::future::{InstrumentedFuture, InstrumentedFutureContext};
use lambda_runtime::LambdaInvocation;
use opentelemetry_sdk::trace::TracerProvider;
use std::{
    sync::Arc,
    task::{Context as TaskContext, Poll},
};
use tower::{Layer, Service};
use tracing::{instrument::Instrumented, Instrument};

use super::context::LambdaServiceContext;

pub struct OtelLambdaLayer<C> {
    context: Arc<C>,
    provider: TracerProvider,
}

impl<C> OtelLambdaLayer<C> {
    pub fn with_context(context: C, provider: TracerProvider) -> Self {
        Self {
            context: Arc::new(context),
            provider,
        }
    }
}

impl<S, C> Layer<S> for OtelLambdaLayer<C> {
    type Service = OtelLambdaService<S, C>;

    fn layer(&self, inner: S) -> Self::Service {
        OtelLambdaService {
            inner,
            context: self.context.clone(),
            provider: self.provider.clone(),
            coldstart: true,
        }
    }
}

impl<T> InstrumentedFutureContext<T> for TracerProvider {
    fn on_result(self, _: &T) {
        self.force_flush();
    }
}

pub struct OtelLambdaService<S, C> {
    inner: S,
    context: Arc<C>,
    provider: TracerProvider,
    coldstart: bool,
}

impl<S, R, C> Service<LambdaInvocation> for OtelLambdaService<S, C>
where
    S: Service<LambdaInvocation, Response = R>,
    C: LambdaServiceContext,
{
    type Response = R;
    type Error = S::Error;
    type Future = InstrumentedFuture<Instrumented<S::Future>, TracerProvider>;

    fn poll_ready(&mut self, cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: LambdaInvocation) -> Self::Future {
        let span = self.context.create_span(&req, self.coldstart);

        self.coldstart = false;

        let future = self.inner.call(req).instrument(span);
        InstrumentedFuture::new(future, self.provider.clone())
    }
}
