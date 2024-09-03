use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    http::StatusCode,
    Error, HttpMessage, ResponseError,
};
use once_cell::sync::Lazy;
use std::{
    collections::{HashMap, HashSet},
    sync::{Mutex, RwLock},
    time::Instant,
};
use tracing::{Id, Span};
use tracing_actix_web::{DefaultRootSpanBuilder, root_span};

static REQUEST_TIMINGS: Lazy<Mutex<HashMap<Id, Instant>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static SKIP_HTTP_TRACE_PATHS: Lazy<RwLock<HashSet<String>>> =
    Lazy::new(|| RwLock::new(HashSet::new()));

pub struct CompactRootSpanBuilder;

impl CompactRootSpanBuilder {
    /// Enable to create span for all paths.
    pub fn all_paths() {
        *SKIP_HTTP_TRACE_PATHS.write().unwrap() = HashSet::new()
    }

    /// Skip to create span for some paths.
    pub fn skip_paths<S: Into<String>>(
        skip_http_trace_paths: impl IntoIterator<Item = S>,
    ) {
        *SKIP_HTTP_TRACE_PATHS.write().unwrap() = skip_http_trace_paths
            .into_iter()
            .map(|path| path.into())
            .collect();
    }
}

impl tracing_actix_web::RootSpanBuilder for CompactRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        if SKIP_HTTP_TRACE_PATHS
            .read()
            .unwrap()
            .contains(request.path())
        {
            request
                .extensions_mut()
                .insert(tracing_actix_web::SkipHttpTrace);
        }
        let span = root_span!(
            request,
            duration = tracing::field::Empty,
            unit = tracing::field::Empty
        );
        // Will be none if tracing subscriber is not initialized
        if let Some(span_id) = span.id() {
            REQUEST_TIMINGS
                .lock()
                .unwrap()
                .insert(span_id, Instant::now());
        }
        span
    }

    fn on_request_end<B: MessageBody>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
        // Will be none if tracing subscriber is not initialized
        if let Some(span_id) = span.id() {
            let start = REQUEST_TIMINGS.lock().unwrap().remove(&span_id);
            if let Some(start) = start {
                let duration = Instant::now() - start;
                span.record("duration", duration.as_micros());
                span.record("unit", "microsecond");
            }
        }

        DefaultRootSpanBuilder::on_request_end(span, outcome)
    }
}
