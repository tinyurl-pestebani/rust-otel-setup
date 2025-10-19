use anyhow::{anyhow, Result};

/// Enum representing the possible logging configurations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogConfig {
    /// Loki configuration.
    Loki(LokiConfig),
    /// OTLP configuration.
    OTLP,
    /// Standard output configuration.
    Stdout,
}


/// Enum representing the possible tracing configurations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TraceConfig {
    /// grpc OTLP configuration.
    OTLP(OTLPTraceConfig),
    /// Standard output configuration.
    StdOut,
}


/// Struct for Loki configuration.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LokiConfig {
    /// The URL of the Loki instance.
    pub url: String,
}


/// Struct for OTLP trace configuration.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OTLPTraceConfig {
    /// The endpoint for the OTLP collector.
    pub endpoint: String,
    /// Export traces to GCP via OTLP.
    pub interceptor: OTLPTraceInterceptor,
}

/// Enum representing the possible OTLP trace interceptors.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OTLPTraceInterceptor {
    GCP,
    None,
}

impl LokiConfig {
    /// Creates a new `LokiConfig` from environment variables.
    ///
    /// The `LOKI_URL` environment variable is used to determine the Loki URL.
    /// If `LOKI_URL` is not set, "http://localhost:3100" is used as the default.
    pub fn from_env() -> Result<Self> {
        let url = std::env::var("LOKI_URL").unwrap_or("http://localhost:3100".to_string());
        Ok(LokiConfig { url })
    }
}


impl OTLPTraceInterceptor {
    /// Creates an `OTLPTraceInterceptor` from environment variables.
    ///
    /// The `OTLP_TRACE_INTERCEPTOR` environment variable is used to determine the interceptor type.
    /// Supported values are "gcp" and "none". If not set, "none" is used as the default.
    pub fn from_env() -> Result<Self> {
        Ok(
            match std::env::var("OTLP_TRACE_INTERCEPTOR").unwrap_or("none".to_string()).as_str() {
                "gcp" => OTLPTraceInterceptor::GCP,
                _ => OTLPTraceInterceptor::None,
            }
        )
    }
}


impl LogConfig {
    /// Creates a `LogConfig` from environment variables.
    ///
    /// The `LOG_PROVIDER` environment variable is used to determine the log provider.
    /// The supported values are "loki", "otlp", and "stdout".
    /// If `LOG_PROVIDER` is not set, "stdout" is used as the default.
    ///
    /// If `LOG_PROVIDER` is "loki", the `LOKI_URL` environment variable is used to determine the Loki URL.
    /// If `LOKI_URL` is not set, "http://localhost:3100" is used as the default.
    pub fn from_env() -> Result<Self> {
        match std::env::var("LOG_PROVIDER").unwrap_or("stdout".to_string()).as_str(){
            "loki" => Ok(LogConfig::Loki(LokiConfig::from_env()?)),
            "otlp" => Ok(LogConfig::OTLP),
            "stdout" => Ok(LogConfig::Stdout),
            _ => Err(anyhow!("Unsupported log config or not set")),
        }
    }
}


impl OTLPTraceConfig {
    /// Creates a new `OTLPTraceConfig` from environment variables.
    ///
    /// The `OTEL_EXPORTER_OTLP_ENDPOINT` environment variable is used to determine the OTLP endpoint.
    /// If `OTEL_EXPORTER_OTLP_ENDPOINT` is not set, "http://localhost:4317" is used as the default.
    pub fn from_env() -> Result<Self> {
        let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .unwrap_or("http://localhost:4317".to_string());
        let interceptor = OTLPTraceInterceptor::from_env()?;
        Ok(OTLPTraceConfig { endpoint, interceptor })
    }
}

impl TraceConfig {
    /// Creates a `TraceConfig` from environment variables.
    ///
    /// The `OTEL_EXPORTER_TRACES` environment variable is used to determine the trace exporter.
    /// The supported values are "grpc" and "stdout".
    /// If `OTEL_EXPORTER_TRACES` is not set, "stdout" is used as the default.
    ///
    /// If `OTEL_EXPORTER_TRACES` is "grpc", the `OTEL_EXPORTER_OTLP_ENDPOINT` environment variable is used to determine the OTLP endpoint.
    /// If `OTEL_EXPORTER_OTLP_ENDPOINT` is not set, "http://localhost:4317" is used as the default.
    pub fn from_env() -> Result<Self> {
        match std::env::var("OTEL_EXPORTER_TRACES").unwrap_or("stdout".to_string()).as_str() {
            "grpc" => Ok(TraceConfig::OTLP(OTLPTraceConfig::from_env()?)),
            "stdout" => Ok(TraceConfig::StdOut),
            _ => Err(anyhow!("Unsupported trace config or not set")),
        }
    }
}
