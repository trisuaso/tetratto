mod assets;
mod avif;
mod macros;
mod routes;

use assets::write_assets;
pub use tetratto_core::*;

use axum::{Extension, Router};
use pathbufd::PathBufD;
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

    // ...
    create_dir_if_not_exists!(&config.dirs.media);
    let images_path =
        PathBufD::current().extend(&[config.dirs.media.clone(), "images".to_string()]);
    create_dir_if_not_exists!(&images_path);
    create_dir_if_not_exists!(
        &PathBufD::current().extend(&[config.dirs.media.clone(), "avatars".to_string()])
    );
    create_dir_if_not_exists!(
        &PathBufD::current().extend(&[config.dirs.media.clone(), "banners".to_string()])
    );

    write_template!(images_path->"default-avatar.svg"(assets::DEFAULT_AVATAR));
    write_template!(images_path->"default-banner.svg"(assets::DEFAULT_BANNER));

    // create templates
    let html_path = PathBufD::current().join(&config.dirs.templates);
    write_assets(&html_path);

    // ...
    let app = Router::new()
        .merge(routes::routes(&config))
        .layer(Extension(Arc::new(RwLock::new((
            DataManager::new(config.clone()).await.unwrap(),
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
