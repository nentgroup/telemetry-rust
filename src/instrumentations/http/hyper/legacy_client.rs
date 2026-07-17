use std::error::Error as StdError;
use std::future::Future;

use hyper::{
    Request, Response,
    body::{Body, Incoming},
};
use hyper_util::client::legacy;

use crate::{Context, http, instrumentations::http::client::HttpClientSpanBuilder};

/// A trait for creating instrumented hyper-util legacy clients with
/// OpenTelemetry tracing.
pub trait HyperLegacyClientInstrument
where
    Self: Sized,
{
    /// The legacy client's connector type.
    type Connector;

    /// The legacy client's request body type.
    type Body;

    /// Wraps this client in an [`InstrumentedLegacyClient`] that can be reused
    /// to send traced requests.
    fn instrument(self) -> InstrumentedLegacyClient<Self::Connector, Self::Body>;
}

impl<C, B> HyperLegacyClientInstrument for legacy::Client<C, B> {
    type Connector = C;
    type Body = B;

    fn instrument(self) -> InstrumentedLegacyClient<C, B> {
        InstrumentedLegacyClient::new(self)
    }
}

/// A reusable wrapper around `hyper_util::client::legacy::Client` that records
/// client spans for each request sent through the client.
#[must_use = "Client does nothing until you call request()"]
pub struct InstrumentedLegacyClient<C, B> {
    inner: legacy::Client<C, B>,
    context: Option<Context>,
}

impl<C, B> InstrumentedLegacyClient<C, B> {
    /// Creates a new instrumented legacy hyper client.
    pub fn new(inner: legacy::Client<C, B>) -> Self {
        Self {
            inner,
            context: None,
        }
    }

    /// Sets the OpenTelemetry context for requests sent by this wrapper.
    pub fn context(mut self, context: &Context) -> Self {
        self.context = Some(context.clone());
        self
    }

    /// Sets the optional OpenTelemetry context for requests sent by this wrapper.
    pub fn set_context(mut self, context: Option<&Context>) -> Self {
        self.context = context.cloned();
        self
    }

    /// Returns the wrapped legacy hyper client.
    pub fn into_inner(self) -> legacy::Client<C, B> {
        self.inner
    }
}

impl<C, B> Clone for InstrumentedLegacyClient<C, B>
where
    legacy::Client<C, B>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            context: self.context.clone(),
        }
    }
}

impl<C, B> InstrumentedLegacyClient<C, B>
where
    C: legacy::connect::Connect + Clone + Send + Sync + 'static,
    B: Body + Send + Unpin + 'static,
    B::Data: Send,
    B::Error: Into<Box<dyn StdError + Send + Sync>>,
{
    /// Sends a constructed request and records an outbound HTTP client span
    /// around it.
    pub fn request(
        &self,
        mut request: Request<B>,
    ) -> impl Future<Output = Result<Response<Incoming>, legacy::Error>> + '_ {
        let span_builder = HttpClientSpanBuilder::from_http_request(&request);
        let span = match self.context.as_ref() {
            Some(context) => span_builder.start_with_context(context),
            None => span_builder.start(),
        };

        http::inject_context_on_context(span.context(), request.headers_mut());

        let inner = &self.inner;

        async move {
            let result = inner.request(request).await;
            match &result {
                Ok(response) => {
                    span.end_response(response.status(), response.version(), None)
                }
                Err(error) => span.end_error(hyper_legacy_error_type(error), error),
            }
            result
        }
    }

    /// Sends a GET request to the supplied URI and records an outbound HTTP
    /// client span around it.
    pub fn get(
        &self,
        uri: hyper::Uri,
    ) -> impl Future<Output = Result<Response<Incoming>, legacy::Error>> + '_
    where
        B: Default,
    {
        let mut request = Request::new(B::default());
        *request.uri_mut() = uri;
        self.request(request)
    }
}

fn hyper_legacy_error_type(error: &legacy::Error) -> &'static str {
    if error.is_connect() {
        "connect"
    } else if let Some(hyper_error) = error
        .source()
        .and_then(|source| source.downcast_ref::<hyper::Error>())
    {
        super::hyper_error_type(hyper_error)
    } else {
        "_OTHER"
    }
}
