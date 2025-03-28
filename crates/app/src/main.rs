mod assets;
mod avif;
mod macros;
mod routes;

use assets::{init_dirs, write_assets};
pub use tetratto_core::*;

use axum::{Extension, Router};
use tera::Tera;
use tower_http::trace::{self, TraceLayer};
use tracing::{Level, info};

use std::sync::Arc;
use tokio::sync::RwLock;

pub(crate) type State = Arc<RwLock<(DataManager, Tera)>>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let config = config::Config::get_config();

    // init
    init_dirs(&config).await;
    let html_path = write_assets(&config).await;

    // ...
    let database = DataManager::new(config.clone()).await.unwrap();
    database.init().await.unwrap();

    let app = Router::new()
        .merge(routes::routes(&config))
        .layer(Extension(Arc::new(RwLock::new((
            database,
            Tera::new(&format!("{html_path}/**/*")).unwrap(),
        )))))
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
