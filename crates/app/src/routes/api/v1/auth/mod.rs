pub mod images;
pub mod ipbans;
pub mod profile;
pub mod social;
pub mod user_warnings;

use super::{LoginProps, RegisterProps};
use crate::{
    State, get_user_from_token,
    model::{ApiReturn, Error, auth::User},
};
use axum::{
    extract::Query,
    http::{HeaderMap, HeaderValue},
    response::{IntoResponse, Redirect},
    Extension, Json,
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use tetratto_shared::hash::hash;

use cf_turnstile::{SiteVerifyRequest, TurnstileClient};

/// `/api/v1/auth/register`
pub async fn register_request(
    headers: HeaderMap,
    // jar: CookieJar,
    Extension(data): Extension<State>,
    Json(props): Json<RegisterProps>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    // let user = get_user_from_token!(jar, data);

    // if user.is_some() {
    //     return (
    //         None,
    //         Json(ApiReturn {
    //             ok: false,
    //             message: Error::AlreadyAuthenticated.to_string(),
    //             payload: (),
    //         }),
    //     );
    // }

    // get real ip
    let real_ip = headers
        .get(data.0.security.real_ip_header.to_owned())
        .unwrap_or(&HeaderValue::from_static(""))
        .to_str()
        .unwrap_or("")
        .to_string();

    // check for ip ban
    if data.get_ipban_by_ip(&real_ip).await.is_ok() {
        return (None, Json(Error::NotAllowed.into()));
    }

    // check captcha
    let client = TurnstileClient::new(data.0.turnstile.secret_key.clone().into());

    let validated = match client
        .siteverify(SiteVerifyRequest {
            response: props.captcha_response,
            ..Default::default()
        })
        .await
    {
        Ok(v) => v,
        Err(e) => return (None, Json(Error::MiscError(e.to_string()).into())),
    };

    if !validated.success | !props.policy_consent {
        return (
            None,
            Json(Error::MiscError("Captcha failed".to_string()).into()),
        );
    }

    // ...
    let mut user = User::new(props.username, props.password);
    user.settings.policy_consent = true;

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
                message: initial_token,
                payload: (),
            }),
        ),
        Err(e) => (None, Json(e.into())),
    }
}

/// `/api/v1/auth/login`
pub async fn login_request(
    headers: HeaderMap,
    // jar: CookieJar,
    Extension(data): Extension<State>,
    Json(props): Json<LoginProps>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    // let user = get_user_from_token!(jar, data);

    // if user.is_some() {
    //     return (None, Json(Error::AlreadyAuthenticated.into()));
    // }

    // get real ip
    let real_ip = headers
        .get(data.0.security.real_ip_header.to_owned())
        .unwrap_or(&HeaderValue::from_static(""))
        .to_str()
        .unwrap_or("")
        .to_string();

    // check for ip ban
    if data.get_ipban_by_ip(&real_ip).await.is_ok() {
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

    // verify totp code
    if !data.check_totp(&user, &props.totp) {
        return (None, Json(Error::NotAllowed.into()));
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

#[derive(Deserialize)]
pub struct SetTokenQuery {
    #[serde(default)]
    pub token: String,
}

/// Set the current user token.
pub async fn set_token_request(Query(props): Query<SetTokenQuery>) -> impl IntoResponse {
    (
        {
            let mut headers = HeaderMap::new();

            headers.insert(
                "Set-Cookie",
                format!(
                    "__Secure-atto-token={}; SameSite=Lax; Secure; Path=/; HostOnly=true; HttpOnly=true; Max-Age={}",
                    props.token,
                    60* 60 * 24 * 365
                )
                .parse()
                .unwrap(),
            );

            headers
        },
        Redirect::to("/"),
    )
}
