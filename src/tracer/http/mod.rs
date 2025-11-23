use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use hyper_util::client::legacy::connect::{Connect, HttpConnector};
use opentelemetry_http::{Bytes, HttpClient, HttpError, Request, Response};
use opentelemetry_http::hyper::HyperClient;
use opentelemetry_otlp::{SpanExporter, WithHttpConfig, WithExportConfig};
use opentelemetry_sdk::trace::TraceError;
use opentelemetry_sdk::trace::SdkTracerProvider as SDKTracerProvider;
use anyhow::Result;
use tokio::runtime::Runtime;
use tonic::codegen::http::HeaderName;
use crate::auth::GetToken;
use crate::config::OTLPTraceConfig;
use crate::resource::get_resource;


/// A Hyper-based HTTP client that adds authentication tokens to requests.
#[derive(Debug, Clone)]
pub struct HyperTracerClient<HttpConnector: Clone+Send+Sync+Connect+'static>{
    client: HyperClient<HttpConnector>,
    token_provider: Arc<dyn GetToken>,
}

/// Implementation of HyperTracerClient
impl HyperTracerClient<HttpConnector> {
    /// Creates a new instance of `HyperTracerClient`.
    /// # Arguments
    /// * `client` - A `HyperClient<HttpConnector>` to send HTTP requests.
    /// * `token_provider` - An `Arc<dyn GetToken>` to provide access tokens.
    /// # Returns
    /// A new `HyperTracerClient` instance.
    pub async fn new(client: HyperClient<HttpConnector>, token_provider: Arc<dyn GetToken>) -> Result<Self> {
        Ok( Self{ client, token_provider })
    }

    /// Adds an authorization token to the request if available.
    /// # Arguments
    /// * `request` - The original HTTP request.
    /// # Returns
    /// The modified HTTP request with the authorization header if a token is available.
    async fn get_token(&self, request: Request<Bytes>) -> Result<Request<Bytes>> {
        let rt = Runtime::new()?;
        let headers = rt.block_on(async { self.token_provider.get_auth_headers().await})?;
        let (mut parts, bts) = request.into_parts();
        for (key, value) in headers {
            let hn = HeaderName::from_str(key.as_str())?;
            parts.headers.insert(hn, value.parse()?);
        }
        let req = Request::from_parts(parts, bts);
        Ok(req)
    }
}


/// Implementation of the HttpClient trait for HyperTracerClient
#[async_trait]
impl HttpClient for HyperTracerClient<HttpConnector> {
    async fn send_bytes(&self, request: Request<Bytes>) -> Result<Response<Bytes>, HttpError> {
        let rt = Runtime::new()?;
        let request = self.get_token(request).await?;
        rt.block_on(async { self.client.send_bytes(request).await})
    }
}


/// Initializes the OTLP HTTP tracer provider with authentication.
/// # Arguments
/// * `otlp_config` - The OTLP trace configuration.
/// * `service_name` - The name of the service.
/// * `token_provider` - An `Arc<dyn GetToken>` to provide access tokens.
/// # Returns
/// A `Result` containing the initialized `SDKTracerProvider` or a `TraceError`.
pub async fn get_http_tracer_provider(otlp_config: &OTLPTraceConfig, service_name: &String, token_provider: Arc<dyn GetToken>) -> Result<SDKTracerProvider, TraceError> {
    let hyper_tracer_client = HyperTracerClient::new(
        HyperClient::with_default_connector(Duration::from_secs(5), None),
        token_provider,
    ).await.map_err(|err| TraceError::from(err.to_string()))?;

    let span_exporter = SpanExporter::builder()
            .with_http()
            .with_endpoint(otlp_config.endpoint.clone())
            .with_http_client(hyper_tracer_client)
            .build()
            .map_err(|err| TraceError::from(err.to_string()))?;

    Ok(
        SDKTracerProvider::builder()
            .with_resource(get_resource(service_name))
            .with_batch_exporter(span_exporter)
            .build()
    )
}
