use crate::{
    State, get_user_from_token,
    model::{ApiReturn, Error},
    routes::api::v1::CreateIpBan,
};
use axum::{Extension, Json, extract::Path, response::IntoResponse};
use axum_extra::extract::CookieJar;
use tetratto_core::model::{auth::IpBan, permissions::FinePermission};

/// Create a new IP ban.
pub async fn create_request(
    jar: CookieJar,
    Path(ip): Path<String>,
    Extension(data): Extension<State>,
    Json(req): Json<CreateIpBan>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    if !user.permissions.check(FinePermission::MANAGE_BANS) {
        return Json(Error::NotAllowed.into());
    }

    match data.create_ipban(IpBan::new(ip, user.id, req.reason)).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "IP ban deleted".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
    }
}

/// Delete the given IP ban.
pub async fn delete_request(
    jar: CookieJar,
    Path(id): Path<usize>,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    if !user.permissions.check(FinePermission::MANAGE_BANS) {
        return Json(Error::NotAllowed.into());
    }

    match data.delete_ipban(id, user).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "IP ban deleted".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
    }
}
