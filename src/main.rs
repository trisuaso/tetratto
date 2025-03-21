mod config;
mod data;
mod routes;

use data::DataManager;

use axum::{Extension, Router};
use tower_http::trace::{self, TraceLayer};
use tracing::{Level, info};

use std::sync::Arc;
use tokio::sync::RwLock;

pub(crate) type State = Arc<RwLock<DataManager>>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let config = config::Config::get_config();

    let app = Router::new()
        .merge(routes::routes())
        .layer(Extension(Arc::new(RwLock::new(
            DataManager::new(config.clone()).await.unwrap(),
        ))))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .unwrap();

    info!("🐐 tetratto.");
    info!("listening on http://0.0.0.0:{}", config.port);
    axum::serve(listener, app).await.unwrap();
}
