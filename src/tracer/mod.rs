pub mod http;
pub mod stdout;
pub mod grpc;
mod reqwest;

use opentelemetry_sdk::trace::TraceError;
use opentelemetry_sdk::trace::SdkTracerProvider as SDKTracerProvider;
use crate::auth::layer;
use crate::config::TraceConfig;

/// Returns the tracer provider based on the provided configuration.
///
/// # Arguments
///
/// * `trace_config` - The tracing configuration.
/// * `service_name` - The name of the service.
pub async fn get_tracer_provider(trace_config: &TraceConfig, service_name: &String) -> Result<SDKTracerProvider, TraceError> {
    match trace_config {
        TraceConfig::HTTP(otlp_config) => {
            let token_provider = layer::new_gen_token(&otlp_config.auth_config);
            http::get_http_tracer_provider(otlp_config, service_name, token_provider).await
        },
        TraceConfig::GRPC(otlp_config) => {
            let token_provider = layer::new_gen_token(&otlp_config.auth_config);
            grpc::init_grpc_otlp_tracer_provider(otlp_config, service_name, token_provider).await
        },
        TraceConfig::REQWEST(otlp_config) => {
            let token_provider = layer::new_gen_token(&otlp_config.auth_config);
            reqwest::get_reqwest_tracer_provider(otlp_config, service_name, token_provider).await
        }
        TraceConfig::StdOut => stdout::get_stdout_tracer_provider().await,
    }
}
