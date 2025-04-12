pub mod auth;
pub mod communities;
pub mod notifications;
pub mod reactions;
pub mod reports;
pub mod util;

use axum::{
    Router,
    routing::{delete, get, post},
};
use serde::Deserialize;
use tetratto_core::model::{
    communities::{
        CommunityContext, CommunityJoinAccess, CommunityReadAccess, CommunityWriteAccess,
        PostContext,
    },
    communities_permissions::CommunityPermission,
    permissions::FinePermission,
    reactions::AssetType,
};

pub fn routes() -> Router {
    Router::new()
        // misc
        .route("/util/proxy", get(util::proxy_request))
        .route("/util/lang", get(util::set_langfile_request))
        // reactions
        .route("/reactions", post(reactions::create_request))
        .route("/reactions/{id}", get(reactions::get_request))
        .route("/reactions/{id}", delete(reactions::delete_request))
        // communities
        .route(
            "/communities/find/{id}",
            get(communities::communities::redirect_from_id),
        )
        .route(
            "/communities",
            post(communities::communities::create_request),
        )
        .route(
            "/communities/{id}",
            delete(communities::communities::delete_request),
        )
        .route(
            "/communities/{id}/title",
            post(communities::communities::update_title_request),
        )
        .route(
            "/communities/{id}/context",
            post(communities::communities::update_context_request),
        )
        .route(
            "/communities/{id}/access/read",
            post(communities::communities::update_read_access_request),
        )
        .route(
            "/communities/{id}/access/write",
            post(communities::communities::update_write_access_request),
        )
        .route(
            "/communities/{id}/access/join",
            post(communities::communities::update_join_access_request),
        )
        .route(
            "/communities/{id}/upload/avatar",
            post(communities::images::upload_avatar_request),
        )
        .route(
            "/communities/{id}/upload/banner",
            post(communities::images::upload_banner_request),
        )
        .route(
            "/communities/{id}/avatar",
            get(communities::images::avatar_request),
        )
        .route(
            "/communities/{id}/banner",
            get(communities::images::banner_request),
        )
        // posts
        .route("/posts", post(communities::posts::create_request))
        .route("/posts/{id}", delete(communities::posts::delete_request))
        .route(
            "/posts/{id}/repost",
            post(communities::posts::create_repost_request),
        )
        .route(
            "/posts/{id}/content",
            post(communities::posts::update_content_request),
        )
        .route(
            "/posts/{id}/context",
            post(communities::posts::update_context_request),
        )
        // auth
        // global
        .route("/auth/register", post(auth::register_request))
        .route("/auth/login", post(auth::login_request))
        .route("/auth/logout", post(auth::logout_request))
        .route("/auth/token", get(auth::set_token_request))
        .route(
            "/auth/upload/avatar",
            post(auth::images::upload_avatar_request),
        )
        .route(
            "/auth/upload/banner",
            post(auth::images::upload_banner_request),
        )
        // profile
        .route("/auth/user/{id}/avatar", get(auth::images::avatar_request))
        .route("/auth/user/{id}/banner", get(auth::images::banner_request))
        .route("/auth/user/{id}/follow", post(auth::social::follow_request))
        .route("/auth/user/{id}/block", post(auth::social::block_request))
        .route(
            "/auth/user/{id}/settings",
            post(auth::profile::update_user_settings_request),
        )
        .route(
            "/auth/user/{id}/role",
            post(auth::profile::update_user_role_request),
        )
        .route(
            "/auth/user/{id}",
            delete(auth::profile::delete_user_request),
        )
        .route(
            "/auth/user/{id}/password",
            post(auth::profile::update_user_password_request),
        )
        .route(
            "/auth/user/{id}/username",
            post(auth::profile::update_user_username_request),
        )
        .route(
            "/auth/user/{id}/tokens",
            post(auth::profile::update_user_tokens_request),
        )
        .route(
            "/auth/user/{id}/verified",
            post(auth::profile::update_user_is_verified_request),
        )
        .route(
            "/auth/user/{id}/totp",
            post(auth::profile::enable_totp_request),
        )
        .route(
            "/auth/user/{id}/totp",
            delete(auth::profile::disable_totp_request),
        )
        .route(
            "/auth/user/{id}/totp/codes",
            post(auth::profile::refresh_totp_codes_request),
        )
        .route(
            "/auth/user/{username}/totp/check",
            get(auth::profile::has_totp_enabled_request),
        )
        .route("/auth/user/me/seen", post(auth::profile::seen_request))
        .route("/auth/user/find/{id}", get(auth::profile::redirect_from_id))
        .route(
            "/auth/user/find_by_ip/{ip}",
            get(auth::profile::redirect_from_ip),
        )
        // warnings
        .route("/warnings/{id}", post(auth::user_warnings::create_request))
        .route(
            "/warnings/{id}",
            delete(auth::user_warnings::delete_request),
        )
        // notifications
        .route(
            "/notifications/my",
            delete(notifications::delete_all_request),
        )
        .route("/notifications/{id}", delete(notifications::delete_request))
        .route(
            "/notifications/{id}/read_status",
            post(notifications::update_read_status_request),
        )
        // community memberships
        .route(
            "/communities/{id}/join",
            post(communities::communities::create_membership),
        )
        .route(
            "/communities/{cid}/memberships/{uid}",
            get(communities::communities::get_membership),
        )
        .route(
            "/communities/{cid}/memberships/{uid}",
            delete(communities::communities::delete_membership),
        )
        .route(
            "/communities/{cid}/memberships/{uid}/role",
            post(communities::communities::update_membership_role),
        )
        // ipbans
        .route("/bans/{ip}", post(auth::ipbans::create_request))
        .route("/bans/id/{id}", delete(auth::ipbans::delete_request))
        // reports
        .route("/reports", post(reports::create_request))
        .route("/reports/{id}", delete(reports::delete_request))
}

#[derive(Deserialize)]
pub struct LoginProps {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub totp: String,
}

#[derive(Deserialize)]
pub struct RegisterProps {
    pub username: String,
    pub password: String,
    pub policy_consent: bool,
    pub captcha_response: String,
}

#[derive(Deserialize)]
pub struct CreateCommunity {
    pub title: String,
}

#[derive(Deserialize)]
pub struct UpdateCommunityTitle {
    pub title: String,
}

#[derive(Deserialize)]
pub struct UpdateCommunityContext {
    pub context: CommunityContext,
}

#[derive(Deserialize)]
pub struct UpdateCommunityReadAccess {
    pub access: CommunityReadAccess,
}

#[derive(Deserialize)]
pub struct UpdateCommunityWriteAccess {
    pub access: CommunityWriteAccess,
}

#[derive(Deserialize)]
pub struct UpdateCommunityJoinAccess {
    pub access: CommunityJoinAccess,
}

#[derive(Deserialize)]
pub struct CreatePost {
    pub content: String,
    pub community: String,
    #[serde(default)]
    pub replying_to: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateRepost {
    pub content: String,
    pub community: String,
}

#[derive(Deserialize)]
pub struct UpdatePostContent {
    pub content: String,
}

#[derive(Deserialize)]
pub struct UpdatePostContext {
    pub context: PostContext,
}

#[derive(Deserialize)]
pub struct CreateReaction {
    pub asset: String,
    pub asset_type: AssetType,
    pub is_like: bool,
}

#[derive(Deserialize)]
pub struct CreateReport {
    pub content: String,
    pub asset: String,
    pub asset_type: AssetType,
}

#[derive(Deserialize)]
pub struct UpdateUserPassword {
    pub from: String,
    pub to: String,
}

#[derive(Deserialize)]
pub struct UpdateUserUsername {
    pub to: String,
}

#[derive(Deserialize)]
pub struct UpdateUserIsVerified {
    pub is_verified: bool,
}

#[derive(Deserialize)]
pub struct UpdateNotificationRead {
    pub read: bool,
}

#[derive(Deserialize)]
pub struct UpdateMembershipRole {
    pub role: CommunityPermission,
}

#[derive(Deserialize)]
pub struct UpdateUserRole {
    pub role: FinePermission,
}

#[derive(Deserialize)]
pub struct DeleteUser {
    pub password: String,
}

#[derive(Deserialize)]
pub struct CreateIpBan {
    pub reason: String,
}

#[derive(Deserialize)]
pub struct DisableTotp {
    pub totp: String,
}

#[derive(Deserialize)]
pub struct CreateUserWarning {
    pub content: String,
}
