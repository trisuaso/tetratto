use crate::{
    State, get_user_from_token,
    model::{ApiReturn, Error},
    routes::api::v1::{UpdateUserIsVerified, UpdateUserPassword, UpdateUserUsername},
};
use axum::{
    Extension, Json,
    extract::Path,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::CookieJar;
use tetratto_core::model::{
    auth::{Token, UserSettings},
    permissions::FinePermission,
};

pub async fn redirect_from_id(
    Extension(data): Extension<State>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match (data.read().await).0
        .get_user_by_id(match id.parse::<usize>() {
            Ok(id) => id,
            Err(_) => return Redirect::to("/"),
        })
        .await
    {
        Ok(u) => Redirect::to(&format!("/user/{}", u.username)),
        Err(_) => Redirect::to("/"),
    }
}

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

    if user.id != id && !user.permissions.check(FinePermission::MANAGE_USERS) {
        return Json(Error::NotAllowed.into());
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

/// Update the password of the given user.
pub async fn update_profile_password_request(
    jar: CookieJar,
    Path(id): Path<usize>,
    Extension(data): Extension<State>,
    Json(req): Json<UpdateUserPassword>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    if user.id != id && !user.permissions.check(FinePermission::MANAGE_USERS) {
        return Json(Error::NotAllowed.into());
    }

    match data
        .update_user_password(id, req.from, req.to, user, false)
        .await
    {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Password updated".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
    }
}

pub async fn update_profile_username_request(
    jar: CookieJar,
    Path(id): Path<usize>,
    Extension(data): Extension<State>,
    Json(req): Json<UpdateUserUsername>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    if user.id != id && !user.permissions.check(FinePermission::MANAGE_USERS) {
        return Json(Error::NotAllowed.into());
    }

    if data.get_user_by_username(&req.to).await.is_ok() {
        return Json(Error::UsernameInUse.into());
    }

    match data.update_user_username(id, req.to, user).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Username updated".to_string(),
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

    if user.id != id && !user.permissions.check(FinePermission::MANAGE_USERS) {
        return Json(Error::NotAllowed.into());
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
