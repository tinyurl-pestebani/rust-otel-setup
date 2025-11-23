pub mod interceptor;

use std::sync::Arc;
use opentelemetry_otlp::{SpanExporter, WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::trace::TraceError;
use tonic::transport::ClientTlsConfig;
use crate::config::OTLPTraceConfig;use opentelemetry_sdk::trace::SdkTracerProvider as SDKTracerProvider;
use crate::auth::GetToken;
use crate::resource::get_resource;


/// Initializes the OTLP tracer provider.
pub async fn init_grpc_otlp_tracer_provider(otlp_config: &OTLPTraceConfig, service_name: &String, token_provider: Arc<dyn GetToken>) -> Result<SDKTracerProvider, TraceError> {
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_config.endpoint.clone())
        .with_tls_config(ClientTlsConfig::new().with_native_roots())
        .with_interceptor(interceptor::TonicInterceptor::new(token_provider))
        .build()
        .map_err(|err| TraceError::from(err.to_string()))?;

    Ok(SDKTracerProvider::builder()
        .with_resource(get_resource(service_name))
        .with_batch_exporter(exporter)
        .build())
}





