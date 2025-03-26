pub mod images;
pub mod social;

use super::AuthProps;
use crate::{
    State, get_user_from_token,
    model::{ApiReturn, Error, auth::User},
};
use axum::{
    Extension, Json,
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use tetratto_shared::hash::hash;

/// `/api/v1/auth/register`
pub async fn register_request(
    headers: HeaderMap,
    jar: CookieJar,
    Extension(data): Extension<State>,
    Json(props): Json<AuthProps>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = get_user_from_token!(jar, data);

    if user.is_some() {
        return (
            None,
            Json(ApiReturn {
                ok: false,
                message: Error::AlreadyAuthenticated.to_string(),
                payload: (),
            }),
        );
    }

    // get real ip
    let real_ip = headers
        .get(data.0.security.real_ip_header.to_owned())
        .unwrap_or(&HeaderValue::from_static(""))
        .to_str()
        .unwrap_or("")
        .to_string();

    // check for ip ban
    if let Ok(_) = data.get_ipban_by_ip(&real_ip).await {
        return (None, Json(Error::NotAllowed.into()));
    }

    // ...
    let mut user = User::new(props.username, props.password);
    let (initial_token, t) = User::create_token(&real_ip);
    user.tokens.push(t);

    // return
    match data.create_user(user).await {
        Ok(_) => (
            Some([(
                "Set-Cookie",
                format!(
                    "__Secure-atto-token={}; SameSite=Lax; Secure; Path=/; HostOnly=true; HttpOnly=true; Max-Age={}",
                    initial_token,
                    60 * 60 * 24 * 365
                ),
            )]),
            Json(ApiReturn {
                ok: true,
                message: "User created".to_string(),
                payload: (),
            }),
        ),
        Err(e) => (None, Json(e.into())),
    }
}

/// `/api/v1/auth/login`
pub async fn login_request(
    headers: HeaderMap,
    jar: CookieJar,
    Extension(data): Extension<State>,
    Json(props): Json<AuthProps>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = get_user_from_token!(jar, data);

    if user.is_some() {
        return (None, Json(Error::AlreadyAuthenticated.into()));
    }

    // get real ip
    let real_ip = headers
        .get(data.0.security.real_ip_header.to_owned())
        .unwrap_or(&HeaderValue::from_static(""))
        .to_str()
        .unwrap_or("")
        .to_string();

    // check for ip ban
    if let Ok(_) = data.get_ipban_by_ip(&real_ip).await {
        return (None, Json(Error::NotAllowed.into()));
    }

    // verify password
    let user = match data.get_user_by_username(&props.username).await {
        Ok(ua) => ua,
        Err(_) => return (None, Json(Error::IncorrectPassword.into())),
    };

    if !user.check_password(props.password) {
        return (None, Json(Error::IncorrectPassword.into()));
    }

    // update tokens
    let mut new_tokens = user.tokens.clone();
    let (unhashed_token_id, token) = User::create_token(&real_ip);
    new_tokens.push(token);

    if let Err(e) = data.update_user_tokens(user.id, new_tokens).await {
        return (None, Json(e.into()));
    }

    // ...
    (
        Some([(
            "Set-Cookie",
            format!(
                "__Secure-atto-token={}; SameSite=Lax; Secure; Path=/; HostOnly=true; HttpOnly=true; Max-Age={}",
                unhashed_token_id,
                60 * 60 * 24 * 365
            ),
        )]),
        Json(ApiReturn {
            ok: true,
            message: unhashed_token_id,
            payload: (),
        }),
    )
}

/// `/api/v1/auth/logout`
pub async fn logout_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return (None, Json(Error::NotAllowed.into())),
    };

    // update tokens
    let token = jar
        .get("__Secure-atto-token")
        .unwrap()
        .to_string()
        .replace("__Secure-atto-token=", "");

    let mut new_tokens = user.tokens.clone();
    new_tokens.remove(
        new_tokens
            .iter()
            .position(|t| t.1 == hash(token.to_string()))
            .unwrap(),
    );

    if let Err(e) = data.update_user_tokens(user.id, new_tokens).await {
        return (None, Json(e.into()));
    }

    // ...
    (
        Some([(
            "Set-Cookie",
            format!(
                "__Secure-atto-token={}; SameSite=Lax; Secure; Path=/; HostOnly=true; HttpOnly=true; Max-Age=0",
                "refresh",
            ),
        )]),
        Json(ApiReturn {
            ok: true,
            message: "Goodbye!".to_string(),
            payload: (),
        }),
    )
}
