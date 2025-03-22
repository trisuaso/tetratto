pub mod api;
pub mod assets;

use crate::{State, get_user_from_token};
use axum::{
    Extension, Router,
    response::{Html, IntoResponse, Redirect},
    routing::get,
};
use axum_extra::extract::CookieJar;

/// `/`
pub async fn index_request(jar: CookieJar, Extension(data): Extension<State>) -> impl IntoResponse {
    let data = data.read().await;
    let user = get_user_from_token!((jar, data) <optional>);

    let mut context = data.initial_context();
    Html(data.1.render("index.html", &mut context).unwrap())
}

/// `/_atto/login`
pub async fn login_request(jar: CookieJar, Extension(data): Extension<State>) -> impl IntoResponse {
    let data = data.read().await;
    let user = get_user_from_token!((jar, data) <optional>);

    if user.is_some() {
        return Err(Redirect::to("/"));
    }

    let mut context = data.initial_context();
    Ok(Html(
        data.1
            .render("_atto/auth/login.html", &mut context)
            .unwrap(),
    ))
}

/// `/_atto/register`
pub async fn register_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = data.read().await;
    let user = get_user_from_token!((jar, data) <optional>);

    if user.is_some() {
        return Err(Redirect::to("/"));
    }

    let mut context = data.initial_context();
    Ok(Html(
        data.1
            .render("_atto/auth/register.html", &mut context)
            .unwrap(),
    ))
}

pub fn routes() -> Router {
    Router::new()
        // assets
        .route("/css/style.css", get(assets::style_css_request))
        .route("/js/atto.js", get(assets::atto_js_request))
        // api
        .nest("/api/v1", api::v1::routes())
        // pages
        .route("/", get(index_request))
        .route("/_atto/login", get(login_request))
        .route("/_atto/register", get(register_request))
}
