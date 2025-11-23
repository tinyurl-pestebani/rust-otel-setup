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
    /// HTTP OTLP configuration.
    HTTP(OTLPTraceConfig),
    /// gRPC OTLP configuration.
    GRPC(OTLPTraceConfig),
    /// gRPC OTLP configuration.
    REQWEST(OTLPTraceConfig),
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
    /// Authorization configuration.
    pub auth_config: AuthConfig,
}

/// Enum representing the possible authentication configurations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AuthConfig {
    /// GCP authentication.
    GCPAuth(GCPAuthConfig),
    /// No authentication.
    Unauthenticated,
}


/// Struct for GCP authentication configuration.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GCPAuthConfig {
    /// Google Cloud Project ID.
    pub project_id: String,
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


impl AuthConfig {
    /// Creates an `AuthConfig` from environment variables.
    ///
    /// The `AUTH_PROVIDER` environment variable is used to determine the authentication provider.
    /// Supported values are "gcp" and "unauthenticated". If not set, "unauthenticated" is used as the default.
    pub fn from_env() -> Result<Self> {
        match std::env::var("AUTH_PROVIDER").unwrap_or("unauthenticated".to_string()).as_str() {
            "gcp" => Ok(AuthConfig::GCPAuth(GCPAuthConfig::from_env()?)),
            _ => Ok(AuthConfig::Unauthenticated),
        }
    }
}


impl GCPAuthConfig {
    /// Creates a new `GCPAuthConfig` from environment variables.
    ///
    /// The `GOOGLE_PROJECT_ID` environment variable is used to determine the GCP project ID.
    /// If `GOOGLE_PROJECT_ID` is not set, an error is returned.
    pub fn from_env() -> Result<Self> {
        let project_id = std::env::var("GOOGLE_PROJECT_ID")
            .map_err(|_| anyhow!("GOOGLE_PROJECT_ID environment variable not set"))?;
        Ok(GCPAuthConfig { project_id })
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
        let auth_config = AuthConfig::from_env()?;
        Ok(OTLPTraceConfig { endpoint, auth_config })
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
            "grpc" => Ok(TraceConfig::GRPC(OTLPTraceConfig::from_env()?)),
            "http" => Ok(TraceConfig::HTTP(OTLPTraceConfig::from_env()?)),
            "reqwest" => Ok(TraceConfig::REQWEST(OTLPTraceConfig::from_env()?)),
            "stdout" => Ok(TraceConfig::StdOut),
            _ => Err(anyhow!("Unsupported trace config or not set")),
        }
    }
}
