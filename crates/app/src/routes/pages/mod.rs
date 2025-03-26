pub mod auth;
pub mod misc;
pub mod profile;

use axum::{Router, routing::get};

pub fn routes() -> Router {
    Router::new()
        // misc
        .route("/", get(misc::index_request))
        // auth
        .route("/auth/register", get(auth::register_request))
        .route("/auth/login", get(auth::login_request))
        // profile
        .route("/user/{username}", get(profile::posts_request))
}
