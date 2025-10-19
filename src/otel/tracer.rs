use opentelemetry_sdk::trace::TraceError;
use opentelemetry_otlp::{SpanExporter, WithExportConfig, WithTonicConfig};
use opentelemetry_otlp::tonic_types::transport::ClientTlsConfig;
use opentelemetry_sdk::trace::SdkTracerProvider as SDKTracerProvider;
use opentelemetry_stdout as stdout;
use crate::authentication::CommonInterceptor;
use crate::config::{OTLPTraceConfig, TraceConfig};
use crate::otel::resource::get_resource;

/// Initializes the OTLP tracer provider.
async fn init_otlp_tracer_provider(otlp_config: &OTLPTraceConfig, service_name: &String) -> Result<SDKTracerProvider, TraceError> {
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_config.endpoint.clone())
        .with_tls_config(ClientTlsConfig::new().with_native_roots())
        .with_interceptor(CommonInterceptor::new(&otlp_config.interceptor))
        .build()
        .map_err(|err| TraceError::from(err.to_string()))?;

    Ok(SDKTracerProvider::builder()
        .with_resource(get_resource(service_name))
        .with_batch_exporter(exporter)
        .build())
}


/// Initializes the SDK tracer provider with a simple exporter to standard output.
fn init_sdk_tracer_provider() -> Result<SDKTracerProvider, TraceError> {
    Ok(SDKTracerProvider::builder()
        .with_simple_exporter(stdout::SpanExporter::default())
        .build())
}

/// Returns the tracer provider based on the provided configuration.
///
/// # Arguments
///
/// * `trace_config` - The tracing configuration.
/// * `service_name` - The name of the service.
pub async fn get_tracer_provider(trace_config: &TraceConfig, service_name: &String) -> Result<SDKTracerProvider, TraceError> {
    match trace_config {
        TraceConfig::OTLP(otlp_config) => init_otlp_tracer_provider(otlp_config, service_name).await,
        TraceConfig::StdOut => init_sdk_tracer_provider(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tracer_provider() {
        let otlp_config = OTLPTraceConfig {
            endpoint: "http://localhost:4317".to_string(),
            interceptor: crate::config::OTLPTraceInterceptor::None,
        };
        let result = get_tracer_provider(&TraceConfig::OTLP(otlp_config), &"basic-axum-example".into()).await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_tracer_provider_unknown() {
        unsafe{
            std::env::set_var("OTEL_EXPORTER_TRACES", "unknown");
        }

        let result = init_sdk_tracer_provider();

        assert!(result.is_ok());
    }
}