pub mod auth;
pub mod communities;
pub mod misc;
pub mod mod_panel;
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
        // timelines
        .route("/", get(misc::index_request))
        .route("/popular", get(misc::popular_request))
        .route("/following", get(misc::following_request))
        .route("/all", get(misc::all_request))
        // question timelines
        .route("/questions", get(misc::index_questions_request))
        .route("/popular/questions", get(misc::popular_questions_request))
        .route(
            "/following/questions",
            get(misc::following_questions_request),
        )
        .route("/all/questions", get(misc::all_questions_request))
        // misc
        .route("/notifs", get(misc::notifications_request))
        .route("/requests", get(misc::requests_request))
        .route("/doc/{*file_name}", get(misc::markdown_document_request))
        .fallback_service(get(misc::not_found))
        // mod
        .route("/mod_panel/audit_log", get(mod_panel::audit_log_request))
        .route("/mod_panel/reports", get(mod_panel::reports_request))
        .route(
            "/mod_panel/file_report",
            get(mod_panel::file_report_request),
        )
        .route("/mod_panel/ip_bans", get(mod_panel::ip_bans_request))
        .route(
            "/mod_panel/profile/{id}",
            get(mod_panel::manage_profile_request),
        )
        .route(
            "/mod_panel/profile/{id}/warnings",
            get(mod_panel::manage_profile_warnings_request),
        )
        // auth
        .route("/auth/register", get(auth::register_request))
        .route("/auth/login", get(auth::login_request))
        // profile
        .route("/settings", get(profile::settings_request))
        .route("/@{username}", get(profile::posts_request))
        .route("/@{username}/following", get(profile::following_request))
        .route("/@{username}/followers", get(profile::followers_request))
        // communities
        .route("/communities", get(communities::list_request))
        .route("/communities/search", get(communities::search_request))
        .route(
            "/communities/intents/post",
            get(communities::create_post_request),
        )
        .route("/community/{title}", get(communities::feed_request))
        .route(
            "/community/{title}/questions",
            get(communities::questions_request),
        )
        .route("/community/{id}/manage", get(communities::settings_request))
        .route(
            "/community/{title}/members",
            get(communities::members_request),
        )
        .route("/post/{id}", get(communities::post_request))
        .route("/question/{id}", get(communities::question_request))
}

pub async fn render_error(
    e: Error,
    jar: &CookieJar,
    data: &(DataManager, tera::Tera, reqwest::Client),
    user: &Option<User>,
) -> String {
    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, user).await;
    context.insert("error_text", &e.to_string());
    data.1.render("misc/error.html", &context).unwrap()
}

#[derive(Deserialize)]
pub struct PaginatedQuery {
    #[serde(default)]
    pub page: usize,
}

#[derive(Deserialize)]
pub struct ProfileQuery {
    #[serde(default)]
    pub page: usize,
    #[serde(default)]
    pub warning: bool,
}

#[derive(Deserialize)]
pub struct SearchedQuery {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub page: usize,
}
