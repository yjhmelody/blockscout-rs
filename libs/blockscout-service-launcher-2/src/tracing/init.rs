use super::{TracingFormat, TracingSettings};

use std::marker::Send;
use tracing::Metadata;
use tracing_subscriber::{
    filter::LevelFilter, fmt::format::FmtSpan, layer::SubscriberExt, prelude::*, Layer,
};

pub fn init_logs(
    tracing_settings: &TracingSettings,
) -> Result<(), anyhow::Error> {
    init_logs_with_filter(
        tracing_settings,
        tracing_subscriber::filter::filter_fn(move |_| true),
    )
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

    #[cfg(feature = "actix-request-id")]
    {
        if let TracingFormat::Json = tracing_settings.format {
            layers.push(super::request_id_layer::layer().boxed());
        }
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
