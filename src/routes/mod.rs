pub mod assets;

use crate::State;
use axum::{
    Extension, Router,
    response::{Html, IntoResponse},
    routing::get,
};

/// `/`
pub async fn index_request(Extension(data): Extension<State>) -> impl IntoResponse {
    let data = data.read().await;
    let mut context = data.initial_context();
    Html(data.1.render("index.html", &mut context).unwrap())
}

/// `/_atto/login`
pub async fn login_request(Extension(data): Extension<State>) -> impl IntoResponse {
    let data = data.read().await;
    let mut context = data.initial_context();
    Html(
        data.1
            .render("_atto/auth/login.html", &mut context)
            .unwrap(),
    )
}

/// `/_atto/register`
pub async fn register_request(Extension(data): Extension<State>) -> impl IntoResponse {
    let data = data.read().await;
    let mut context = data.initial_context();
    Html(
        data.1
            .render("_atto/auth/register.html", &mut context)
            .unwrap(),
    )
}

pub fn routes() -> Router {
    Router::new()
        // assets
        .route("/css/style.css", get(assets::style_css_request))
        .route("/js/atto.js", get(assets::atto_js_request))
        // pages
        .route("/", get(index_request))
        .route("/_atto/login", get(login_request))
        .route("/_atto/register", get(register_request))
}
