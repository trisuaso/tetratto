use super::{ApiReturn, AuthProps};
use crate::{
    State,
    data::model::{Error, User},
    get_user_from_token,
};
use axum::{Extension, Json, response::IntoResponse};
use axum_extra::extract::CookieJar;

pub async fn register_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Json(props): Json<AuthProps>,
) -> impl IntoResponse {
    let data = data.read().await;
    let user = get_user_from_token!((jar, data) <optional>);

    if user.is_some() {
        return Json(ApiReturn {
            ok: false,
            message: Error::AlreadyAuthenticated.to_string(),
            payload: (),
        });
    }

    match data
        .create_user(User::new(props.username, props.password))
        .await
    {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "User created".to_string(),
            payload: (),
        }),
        Err(_) => Json(ApiReturn {
            ok: false,
            message: Error::Unknown.to_string(),
            payload: (),
        }),
    }
}
