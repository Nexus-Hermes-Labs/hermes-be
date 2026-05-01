//! Tracing initialisation: structured logs (Loki) + distributed traces (Tempo).
//!
//! Spans created by `tracing` (e.g. by `HermesMakeSpan`) flow through two
//! sinks in parallel:
//!
//! 1. The `fmt` layer renders them as JSON / pretty text for stdout + the
//!    rolling error log file. Promtail tails container stdout into Loki.
//! 2. The `tracing_opentelemetry` layer converts each span into an
//!    OpenTelemetry span, batched by an OTLP exporter pointed at Tempo.
//!
//! The OTel half only activates when `APP_LOGGING__OTLP_ENDPOINT` is set.
//! Local `cargo run` without an OTLP endpoint stays log-only.
//!
//! W3C `traceparent` propagation: this module installs the global
//! [`TraceContextPropagator`]. `HermesMakeSpan` and the gRPC client/server
//! interceptors use the *global* propagator, so context format stays in
//! lockstep with the exporter configuration here.

use anyhow::Result;
use common_config::logging::LogFormat;
use common_config::LoggingConfig;
use opentelemetry::{global, trace::TracerProvider as _, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
    runtime,
    trace::{self as sdktrace, Config as TraceConfig},
    Resource,
};
use tracing_appender::rolling;
use tracing_subscriber::{
    filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry,
};

/// Tracer name used when extracting a tracer from the global provider.
const TRACER_NAME: &str = "hermes";

/// Initialise structured logging plus, when `config.otlp_endpoint` is set,
/// a parallel OTLP span exporter pointed at Tempo.
///
/// Must be called exactly once per process, after the global config is
/// loaded but before any code that emits spans (which is everything).
pub fn init_tracing(config: &LoggingConfig, service_name: &str, environment: &str) -> Result<()> {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));
    let file_appender = rolling::daily("logs", "error.log");

    let otel_tracer = match &config.otlp_endpoint {
        Some(endpoint) if !endpoint.is_empty() => {
            Some(install_otel_pipeline(endpoint, service_name, environment)?)
        }
        _ => None,
    };

    // Build the subscriber separately for each (format, otel?) combination.
    // `Option<OpenTelemetryLayer>` does not survive type-inference through the
    // `.with()` chain — `OpenTelemetryLayer` is parametrised by the subscriber
    // type, so we have to attach it directly to a concrete chain.
    match (config.format.clone(), otel_tracer) {
        (LogFormat::Json, Some(tracer)) => {
            Registry::default()
                .with(env_filter)
                .with(tracing_opentelemetry::layer().with_tracer(tracer))
                .with(json_console_layer())
                .with(json_file_layer(file_appender))
                .init();
        }
        (LogFormat::Json, None) => {
            Registry::default()
                .with(env_filter)
                .with(json_console_layer())
                .with(json_file_layer(file_appender))
                .init();
        }
        (LogFormat::Pretty, Some(tracer)) => {
            Registry::default()
                .with(env_filter)
                .with(tracing_opentelemetry::layer().with_tracer(tracer))
                .with(pretty_console_layer())
                .with(pretty_file_layer(file_appender))
                .init();
        }
        (LogFormat::Pretty, None) => {
            Registry::default()
                .with(env_filter)
                .with(pretty_console_layer())
                .with(pretty_file_layer(file_appender))
                .init();
        }
    }

    tracing::info!(
        service_name = service_name,
        environment = environment,
        format = ?config.format,
        level = config.level,
        otlp_enabled = config.otlp_endpoint.is_some(),
        "Tracing initialized"
    );

    Ok(())
}

fn json_console_layer<S>() -> fmt::Layer<S, fmt::format::JsonFields, fmt::format::Format<fmt::format::Json>>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(true)
        .json()
}

fn json_file_layer<S>(
    appender: tracing_appender::rolling::RollingFileAppender,
) -> tracing_subscriber::filter::Filtered<
    fmt::Layer<
        S,
        fmt::format::JsonFields,
        fmt::format::Format<fmt::format::Json>,
        tracing_appender::rolling::RollingFileAppender,
    >,
    LevelFilter,
    S,
>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    use tracing_subscriber::Layer;
    fmt::layer()
        .with_writer(appender)
        .with_target(true)
        .with_level(true)
        .with_thread_ids(true)
        .json()
        .with_filter(LevelFilter::ERROR)
}

fn pretty_console_layer<S>(
) -> fmt::Layer<S, fmt::format::Pretty, fmt::format::Format<fmt::format::Pretty>>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(true)
        .pretty()
        .with_ansi(true)
}

fn pretty_file_layer<S>(
    appender: tracing_appender::rolling::RollingFileAppender,
) -> tracing_subscriber::filter::Filtered<
    fmt::Layer<
        S,
        fmt::format::Pretty,
        fmt::format::Format<fmt::format::Pretty>,
        tracing_appender::rolling::RollingFileAppender,
    >,
    LevelFilter,
    S,
>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    use tracing_subscriber::Layer;
    fmt::layer()
        .with_writer(appender)
        .with_target(true)
        .with_level(true)
        .with_thread_ids(true)
        .pretty()
        .with_filter(LevelFilter::ERROR)
}

/// Build the OTLP/gRPC pipeline and return a tracer bound to it.
///
/// Side effects: installs the global `TextMapPropagator` so HTTP/gRPC
/// extractors and injectors all use W3C TraceContext, and installs the
/// resulting `TracerProvider` globally.
fn install_otel_pipeline(
    endpoint: &str,
    service_name: &str,
    environment: &str,
) -> Result<sdktrace::Tracer> {
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(endpoint);

    let resource = Resource::new(vec![
        KeyValue::new("service.name", service_name.to_string()),
        KeyValue::new("deployment.environment", environment.to_string()),
    ]);

    let provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(TraceConfig::default().with_resource(resource))
        .install_batch(runtime::Tokio)?;

    let tracer = provider.tracer(TRACER_NAME);

    // W3C `traceparent` is the on-the-wire format. Set globally so both
    // inbound extraction and outbound injection use the same header schema.
    global::set_text_map_propagator(TraceContextPropagator::new());

    Ok(tracer)
}

/// Flush any pending spans and tear down the global tracer provider.
/// Call from each service's main on shutdown — without this, the last
/// batch of spans never reaches Tempo.
pub fn shutdown_tracing() {
    tracing::info!("Shutting down tracing");
    // No-op when OTel was never installed; safe either way.
    global::shutdown_tracer_provider();
}
