use std::str::FromStr;
use std::sync::Arc;
use async_trait::async_trait;
use opentelemetry_http::{Bytes, HttpClient, HttpError, Request, Response};
use opentelemetry_otlp::{SpanExporter, WithHttpConfig, WithExportConfig};
use opentelemetry_sdk::trace::TraceError;
use opentelemetry_sdk::trace::SdkTracerProvider as SDKTracerProvider;
use reqwest;
use reqwest::header::HeaderName;
use tokio::runtime::Runtime;
use crate::auth::GetToken;
use crate::config::OTLPTraceConfig;
use crate::resource::get_resource;

/// A Reqwest-based HTTP client that adds authentication tokens to requests.
#[derive(Debug, Clone)]
pub struct ReqwestTracerClient{
    client: Arc<dyn HttpClient>,
    token_provider: Arc<dyn GetToken>,
}


impl ReqwestTracerClient {
    /// Creates a new instance of `ReqwestTracerClient`.
    /// # Arguments
    /// * `client` - An `Arc<dyn HttpClient>` to send HTTP requests.
    /// * `token_provider` - An `Arc<dyn GetToken>` to provide access tokens.
    /// # Returns
    /// A new `ReqwestTracerClient` instance.
    pub fn new(client: Arc<dyn HttpClient>, token_provider: Arc<dyn GetToken>) -> Self {
        Self { client, token_provider }
    }

    /// Adds an authorization token to the request if available.
    /// # Arguments
    /// * `request` - The original HTTP request.
    /// # Returns
    /// The modified HTTP request with the authorization header if a token is available.
    async fn get_token(&self, request: Request<Bytes>) -> anyhow::Result<Request<Bytes>> {
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


/// Implementation of the HttpClient trait for ReqwestTracerClient
#[async_trait]
impl HttpClient for ReqwestTracerClient {
    async fn send_bytes(&self, request: Request<Bytes>) -> anyhow::Result<Response<Bytes>, HttpError> {
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
pub async fn get_reqwest_tracer_provider(otlp_config: &OTLPTraceConfig, service_name: &String, token_provider: Arc<dyn GetToken>) -> anyhow::Result<SDKTracerProvider, TraceError> {
    let http_client = Arc::new(
        reqwest::Client::builder()
            .build()
            .unwrap_or_default(),
    ) as Arc<dyn HttpClient>;

    let reqwest_tracer_client = ReqwestTracerClient::new(
        // HyperClient::with_default_connector(Duration::from_secs(5), None),
        http_client,
        token_provider,
    );

    let span_exporter = SpanExporter::builder()
        .with_http()
        .with_endpoint(otlp_config.endpoint.clone())
        .with_http_client(reqwest_tracer_client)
        .build()
        .map_err(|err| TraceError::from(err.to_string()))?;

    Ok(
        SDKTracerProvider::builder()
            .with_resource(get_resource(service_name))
            .with_batch_exporter(span_exporter)
            .build()
    )
}


/*
fn some_function() {
    let http_client = Some(Arc::new(
        reqwest::Client::builder()
            .build()
            .unwrap_or_default(),
    ) as Arc<dyn HttpClient>);
}

 */