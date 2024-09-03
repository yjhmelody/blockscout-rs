use super::{OTLPSettings, TracingFormat, TracingSettings};

use std::marker::Send;

use opentelemetry::trace::TraceError;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::{
    trace::{self, RandomIdGenerator, Sampler},
    Resource,
};
use opentelemetry_semantic_conventions as semconv;
use tracing::Metadata;
use tracing_subscriber::{
    filter::LevelFilter, fmt::format::FmtSpan, layer::SubscriberExt, prelude::*, Layer,
};

pub fn init_logs(tracing_settings: &TracingSettings) -> Result<(), anyhow::Error> {
    init_logs_with_filter(
        tracing_settings,
        tracing_subscriber::filter::filter_fn(move |_| true),
    )
}

pub fn init_telemetry(setting: &OTLPSettings) -> Result<TracerProvider, TraceError> {
    let mut exporter = opentelemetry_otlp::new_exporter().tonic();
    exporter = exporter.with_endpoint(setting.agent_endpoint.clone());
    let mut attrs = vec![KeyValue::new(
        semconv::resource::SERVICE_NAME,
        setting.service_name.clone(),
    )];

    if let Some(pod_name) = option_env!("POD_NAME") {
        attrs.push(KeyValue::new(semconv::resource::K8S_POD_NAME, pod_name));
    }

    if let Some(pod_namespace) = option_env!("POD_NAMESPACE") {
        attrs.push(KeyValue::new(
            semconv::resource::K8S_NAMESPACE_NAME,
            pod_namespace,
        ));
    }

    let resource = Resource::from_schema_url(attrs, semconv::SCHEMA_URL);

    let trace_config = trace::Config::default()
        .with_resource(resource)
        .with_sampler(Sampler::AlwaysOn)
        .with_id_generator(RandomIdGenerator::default())
        .with_max_events_per_span(64)
        .with_max_attributes_per_span(16)
        .with_max_events_per_span(16);

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(trace_config)
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    Ok(tracer)
}

pub fn init_logs_with_filter<F: Fn(&Metadata) -> bool + Send + Sync + 'static>(
    tracing_settings: &TracingSettings,
    filter: tracing_subscriber::filter::FilterFn<F>,
) -> Result<(), anyhow::Error> {
    // If tracing is disabled, there is nothing to initialize
    if !tracing_settings.enabled {
        return Ok(());
    }

    let mut layers: Vec<_> = vec![];

    if let TracingFormat::Json = tracing_settings.format {
        layers.push(super::request_id_layer::layer().boxed());
    }

    let stdout_layer: Box<dyn Layer<_> + Sync + Send + 'static> = match tracing_settings.format {
        TracingFormat::Default => tracing_subscriber::fmt::layer()
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_filter(
                tracing_subscriber::EnvFilter::builder()
                    .with_default_directive(LevelFilter::INFO.into())
                    .from_env_lossy(),
            )
            .with_filter(filter)
            .boxed(),
        TracingFormat::Json => tracing_subscriber::fmt::layer()
            .json()
            // .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .flatten_event(true)
            .with_current_span(true)
            .with_span_list(false)
            .with_filter(
                tracing_subscriber::EnvFilter::builder()
                    .with_default_directive(LevelFilter::INFO.into())
                    .from_env_lossy(),
            )
            .with_filter(filter)
            .boxed(),
    };
    layers.push(stdout_layer);

    let registry = tracing_subscriber::registry().with(layers);
    registry.try_init()?;

    Ok(())
}
