//! # rust_otel_setup
//!
//! `rust_otel_setup` is a library for configuring OpenTelemetry logging and tracing in Rust applications.
//! It provides a simple way to set up OpenTelemetry with logging and tracing capabilities.
pub mod otel;
pub mod config;
mod tracer;
mod auth;
pub mod resource;