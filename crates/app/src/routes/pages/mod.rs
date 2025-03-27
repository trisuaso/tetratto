pub mod auth;
pub mod communities;
pub mod misc;
pub mod profile;

use axum::{Router, routing::get};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use tetratto_core::{
    DataManager,
    model::{Error, auth::User},
};

use crate::{assets::initial_context, get_lang};

pub fn routes() -> Router {
    Router::new()
        // misc
        .route("/", get(misc::index_request))
        // auth
        .route("/auth/register", get(auth::register_request))
        .route("/auth/login", get(auth::login_request))
        // profile
        .route("/user/{username}", get(profile::posts_request))
        // communities
        .route("/communities", get(communities::list_request))
}

pub async fn render_error(
    e: Error,
    jar: &CookieJar,
    data: &(DataManager, tera::Tera),
    user: &Option<User>,
) -> String {
    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &user).await;
    context.insert("error_text", &e.to_string());
    data.1.render("misc/error.html", &mut context).unwrap()
}

#[derive(Deserialize)]
pub struct PaginatedQuery {
    #[serde(default)]
    pub page: usize,
}
