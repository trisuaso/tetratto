pub mod auth;
use axum::{
    Router,
    routing::{get, post},
};
use serde::Deserialize;

pub fn routes() -> Router {
    Router::new()
        // auth
        // global
        .route("/auth/register", post(auth::register_request))
        .route("/auth/login", post(auth::login_request))
        .route("/auth/logout", post(auth::logout_request))
        .route(
            "/auth/upload/avatar",
            post(auth::images::upload_avatar_request),
        )
        .route(
            "/auth/upload/banner",
            post(auth::images::upload_banner_request),
        )
        // profile
        .route(
            "/auth/profile/{id}/avatar",
            get(auth::images::avatar_request),
        )
        .route(
            "/auth/profile/{id}/banner",
            get(auth::images::banner_request),
        )
}

#[derive(Deserialize)]
pub struct AuthProps {
    pub username: String,
    pub password: String,
}
