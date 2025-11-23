use opentelemetry_sdk::trace::TraceError;
use opentelemetry_sdk::trace::SdkTracerProvider as SDKTracerProvider;
use opentelemetry_stdout as stdout;

/// Returns a tracer provider that exports spans to standard output.
pub async fn get_stdout_tracer_provider() -> Result<SDKTracerProvider, TraceError> {
    Ok(
        SDKTracerProvider::builder()
            .with_simple_exporter(stdout::SpanExporter::default())
            .build()
    )
}