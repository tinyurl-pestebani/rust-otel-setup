use anyhow::Result;
use opentelemetry_sdk::trace::Tracer;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::LogExporter;
use tracing_loki::BackgroundTask;
use tracing_loki::url::Url;
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::config::{LogConfig, LokiConfig};
use crate::otel::resource::get_resource;

/// Enum representing the possible log layers.
pub enum LogLayer {
    /// Loki log layer.
    Loki(tracing_loki::Layer, BackgroundTask),
    /// OTLP log layer.
    OTLP,
    /// Standard output log layer.
    Stdout,
}

/// Initializes the Loki log provider.
fn init_loki_log_provider(config: &LokiConfig, service_name: &String) -> Result<LogLayer> {
    let (layer, task) = tracing_loki::layer(
        Url::parse(config.url.as_str())?,
        [("service".into(), service_name.into())].into_iter().collect(),
        [].into_iter().collect(),
    )?;
    Ok(LogLayer::Loki(layer, task))
}


/// Returns the log layer based on the provided configuration.
///
/// # Arguments
///
/// * `config` - The logging configuration.
/// * `service_name` - The name of the service.
pub fn get_logger(config: &LogConfig, service_name: &String) -> Result<LogLayer> {
    match config { 
        LogConfig::Loki(loki_config) => init_loki_log_provider(loki_config, service_name),
        LogConfig::OTLP => Ok(LogLayer::OTLP),
        LogConfig::Stdout => Ok(LogLayer::Stdout),
    }
}


/// Sets the global logger.
///
/// # Arguments
///
/// * `log_layer` - The log layer to set.
/// * `tracer` - The tracer to use.
/// * `service_name` - The name of the service.
pub fn set_logger(log_layer: LogLayer, tracer: Tracer, service_name: &String) -> Result<()> {
    let filter = EnvFilter::from_default_env();
    match log_layer {
        LogLayer::Loki(layer, task) =>{
            let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
            tokio::spawn(task);
            tracing_subscriber::registry()
                .with(filter)
                .with(layer)
                .with(telemetry)
                .init();
        },
        LogLayer::OTLP => {
            let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
            let exp = LogExporter::builder().with_http().build().expect("Failed to create OTLP log exporter");
            let prov = SdkLoggerProvider::builder().with_batch_exporter(exp).with_resource(get_resource(service_name)).build();
            let log_layer = OpenTelemetryTracingBridge::new(&prov);
            tracing_subscriber::registry()
                .with(filter)
                .with(telemetry)
                .with(log_layer)
                .init();
        },
        _ => {
            let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt::layer())
                .with(telemetry)
                .init();
        }
    };

    Ok(())
}
