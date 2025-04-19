pub mod api;
pub mod assets;
pub mod pages;

use crate::config::Config;
use axum::{
    Router,
    routing::{get, get_service},
};

pub fn routes(config: &Config) -> Router {
    Router::new()
        // assets
        .route("/css/style.css", get(assets::style_css_request))
        .route("/js/loader.js", get(assets::loader_js_request))
        .route("/js/atto.js", get(assets::atto_js_request))
        .route("/js/me.js", get(assets::me_js_request))
        .nest_service(
            "/public",
            get_service(tower_http::services::ServeDir::new(&config.dirs.assets)),
        )
        .route("/public/favicon.svg", get(assets::favicon_request))
        .route_service(
            "/robots.txt",
            tower_http::services::ServeFile::new(format!("{}/robots.txt", config.dirs.assets)),
        )
        // api
        .nest("/api/v1", api::v1::routes())
        // pages
        .merge(pages::routes())
}
