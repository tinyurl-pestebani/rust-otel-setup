# rust-otel-setup
This package provides a simple way to set up the OpenTelemetry SDK for Rust applications, with support for different trace and log providers based on environment variables.


Variables
---------
- `OTEL_EXPORTER_OTLP_ENDPOINT`: The endpoint for the OTLP exporter. Defaults to `http://localhost:4317`.
- `OTEL_EXPORTER_TRACES`: The exporter type for traces. Defaults to `stdout`. Valid values are `gcp`, `jaeger` and `stdout`.
- `LOG_PROVIDER`: The log provider to use. Defaults to `stdout`. Valid values are `loki`, `otlp`, and `stdout`.
- `LOKI_URL`: The URL for the Loki log provider. Defaults to `http://localhost:3100`.
- `GOOGLE_PROJECT_ID`: ID of the project of GCP. Required if `OTEL_EXPORTER_TRACES` is set to `gcp`.
