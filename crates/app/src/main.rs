mod assets;
mod avif;
mod macros;
mod routes;
mod sanitize;

use assets::{init_dirs, write_assets};
pub use tetratto_core::*;

use axum::{Extension, Router};
use reqwest::Client;
use tera::{Tera, Value};
use tower_http::trace::{self, TraceLayer};
use tracing::{Level, info};

use std::{collections::HashMap, env::var, sync::Arc};
use tokio::sync::RwLock;

pub(crate) type State = Arc<RwLock<(DataManager, Tera, Client)>>;

fn render_markdown(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    Ok(tetratto_shared::markdown::render_markdown(value.as_str().unwrap()).into())
}

fn color_escape(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    Ok(sanitize::color_escape(value.as_str().unwrap()).into())
}

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

    let mut tera = Tera::new(&format!("{html_path}/**/*")).unwrap();
    tera.register_filter("markdown", render_markdown);
    tera.register_filter("color", color_escape);

    let client = Client::new();

    let app = Router::new()
        .merge(routes::routes(&config))
        .layer(Extension(Arc::new(RwLock::new((database, tera, client)))))
        .layer(axum::extract::DefaultBodyLimit::max(
            var("BODY_LIMIT")
                .unwrap_or("8388608".to_string())
                .parse::<usize>()
                .unwrap(),
        ))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .unwrap();

    info!("🐇 tetratto.");
    info!("listening on http://0.0.0.0:{}", config.port);
    axum::serve(listener, app).await.unwrap();
}
