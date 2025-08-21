use opentelemetry_sdk::trace::TraceError;
use opentelemetry_gcloud_trace::GcpCloudTraceExporter;
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::trace::SdkTracerProvider as SDKTracerProvider;
use opentelemetry_stdout as stdout;
use crate::config::{OTLPTraceConfig, TraceConfig};
use crate::otel::resource::get_resource;

/// Initializes the OTLP tracer provider.
fn init_otlp_tracer_provider(otlp_config: &OTLPTraceConfig, service_name: &String) -> Result<SDKTracerProvider, TraceError> {
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_config.endpoint.clone())
        .build()
        .expect("Failed to create span exporter");

    Ok(SDKTracerProvider::builder()
        .with_resource(get_resource(service_name))
        .with_batch_exporter(exporter)
        .build())
}

/// Initializes the GCP tracer provider.
async fn init_gcp_tracer_provider(service_name: &String) -> Result<SDKTracerProvider, TraceError> {
    let google_project_id = std::env::var("GOOGLE_PROJECT_ID").unwrap();
    let exporter = GcpCloudTraceExporter::new(google_project_id.as_str(), get_resource(service_name)).await?;

    Ok(
        opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .build()
    )
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
        TraceConfig::Jaeger(otlp_config) => init_otlp_tracer_provider(otlp_config, service_name),
        TraceConfig::GCP => init_gcp_tracer_provider(service_name).await,
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
        };
        let result = get_tracer_provider(&TraceConfig::Jaeger(otlp_config), &"basic-axum-example".into()).await;

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