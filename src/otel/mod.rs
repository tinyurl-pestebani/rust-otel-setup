//! # OpenTelemetry Module
//!
//! This module provides the main entry point for configuring OpenTelemetry.
mod tracer;
mod logger;
mod resource;

use opentelemetry::trace::TracerProvider;
use crate::otel::logger::{get_logger, set_logger};
use anyhow::Result;
use crate::otel::tracer::get_tracer_provider;

use opentelemetry_sdk::trace::SdkTracerProvider as SDKTracerProvider;
use crate::config::{LogConfig, TraceConfig};


/// The main OpenTelemetry object.
pub struct OpenTelemetryObject {
    /// The tracer provider.
    pub tracer: SDKTracerProvider,
}


impl OpenTelemetryObject {
    /// Creates a new `OpenTelemetryObject`.
    ///
    /// This function initializes the tracer and logger providers based on the provided configuration.
    ///
    /// # Arguments
    ///
    /// * `log_config` - The logging configuration.
    /// * `trace_config` - The tracing configuration.
    /// * `service_name` - The name of the service.
    pub async fn new(log_config: &LogConfig, trace_config: &TraceConfig, service_name: String) -> Result<Self> {
        let exporter = get_tracer_provider(trace_config, &service_name).await?;

        let log_layer = get_logger(log_config, &service_name)?;

        let tracer = exporter.tracer(service_name.clone());

        set_logger(log_layer, tracer, &service_name)?;

        Ok(OpenTelemetryObject { tracer: exporter })
    }

    /// Shuts down the tracer provider.
    pub fn stop(&self) -> Result<()> {
        Ok(self.tracer.shutdown()?)
    }
}

