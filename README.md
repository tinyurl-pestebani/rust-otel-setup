# rust-otel-setup
This package provides a simple way to set up the OpenTelemetry SDK for Rust applications, with support for different trace and log providers based on environment variables.


Variables
---------
- `OTEL_EXPORTER_OTLP_ENDPOINT`: The endpoint for the OTLP exporter. Defaults to `http://localhost:4317`. If exporting to GCP, set this to `https://telemetry.googleapis.com`. If exporting via `reqwest`, this value must end with `/v1/traces`, for example: `https://telemetry.googleapis.com/v1/traces`.
- `OTEL_EXPORTER_TRACES`: The exporter type for traces. Defaults to `stdout`. Valid values are `grpc`, `http`, `reqwest` and `stdout`.
- `LOG_PROVIDER`: The log provider to use. Defaults to `stdout`. Valid values are `loki`, `otlp`, and `stdout`.
- `LOKI_URL`: The URL for the Loki log provider. Defaults to `http://localhost:3100`.
- `OTLP_TRACE_INTERCEPTOR`: The trace interceptor for OTLP exporter. Defaults to `none`. Valid values are `gcp` and `none`.
- `GOOGLE_PROJECT_ID`: ID of the project of GCP. Required if `OTEL_EXPORTER_TRACES` is set to `grpc` and `OTLP_TRACE_INTERCEPTOR` is set to `gcp`.
