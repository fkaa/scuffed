use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use axum::Router;

use hyper::{Body, Request};
use tower_http::trace::TraceLayer;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "scuffed=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

pub fn tracing_layer(router: Router) -> Router {
    let req_id = Arc::new(AtomicU64::new(0));

    let layer = TraceLayer::new_for_http().make_span_with(move |request: &Request<Body>| {
        let req_id = req_id.fetch_add(1, Ordering::SeqCst);

        tracing::debug_span!(
            "request",
            method = %request.method(),
            uri = %request.uri(),
            id = req_id
        )
    });

    router.layer(layer)
}

