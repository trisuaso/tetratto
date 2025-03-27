use crate::{
    State, get_user_from_token,
    model::{ApiReturn, Error},
    routes::api::v1::UpdateUserIsVerified,
};
use axum::{Extension, Json, extract::Path, response::IntoResponse};
use axum_extra::extract::CookieJar;
use tetratto_core::model::{
    auth::{Token, UserSettings},
    permissions::FinePermission,
};

/// Update the settings of the given user.
pub async fn update_profile_settings_request(
    jar: CookieJar,
    Path(id): Path<usize>,
    Extension(data): Extension<State>,
    Json(req): Json<UserSettings>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    if user.id != id {
        if !user.permissions.check(FinePermission::MANAGE_USERS) {
            return Json(Error::NotAllowed.into());
        }
    }

    match data.update_user_settings(id, req).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Settings updated".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
    }
}

/// Update the tokens of the given user.
pub async fn update_profile_tokens_request(
    jar: CookieJar,
    Path(id): Path<usize>,
    Extension(data): Extension<State>,
    Json(req): Json<Vec<Token>>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    if user.id != id {
        if !user.permissions.check(FinePermission::MANAGE_USERS) {
            return Json(Error::NotAllowed.into());
        }
    }

    match data.update_user_tokens(id, req).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Tokens updated".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
    }
}

/// Update the verification status of the given user.
pub async fn update_profile_is_verified_request(
    jar: CookieJar,
    Path(id): Path<usize>,
    Extension(data): Extension<State>,
    Json(req): Json<UpdateUserIsVerified>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data
        .update_user_verified_status(id, req.is_verified, user)
        .await
    {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Verified status updated".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
    }
}
