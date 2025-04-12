use crate::{
    get_user_from_token,
    model::{ApiReturn, Error},
    routes::api::v1::CreateUserWarning,
    State,
};
use axum::{Extension, Json, extract::Path, response::IntoResponse};
use axum_extra::extract::CookieJar;
use tetratto_core::model::{auth::UserWarning, permissions::FinePermission};

/// Create a new user warning.
pub async fn create_request(
    jar: CookieJar,
    Path(uid): Path<usize>,
    Extension(data): Extension<State>,
    Json(req): Json<CreateUserWarning>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    if !user.permissions.check(FinePermission::MANAGE_BANS) {
        return Json(Error::NotAllowed.into());
    }

    match data
        .create_user_warning(UserWarning::new(uid, user.id, req.content))
        .await
    {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "User warning created".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
    }
}

/// Delete the given user warning.
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

    if !user.permissions.check(FinePermission::MANAGE_WARNINGS) {
        return Json(Error::NotAllowed.into());
    }

    match data.delete_user_warning(id, user).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "User warning deleted".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
    }
}
