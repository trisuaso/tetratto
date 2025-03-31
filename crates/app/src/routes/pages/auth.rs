use crate::{State, assets::initial_context, get_lang, get_user_from_token};
use axum::{
    Extension,
    response::{Html, IntoResponse, Redirect},
};
use axum_extra::extract::CookieJar;

/// `/auth/login`
pub async fn login_request(jar: CookieJar, Extension(data): Extension<State>) -> impl IntoResponse {
    let data = data.read().await;
    let user = get_user_from_token!(jar, data.0);

    if user.is_some() {
        return Err(Redirect::to("/"));
    }

    let lang = get_lang!(jar, data.0);
    let context = initial_context(&data.0.0, lang, &user).await;

    Ok(Html(data.1.render("auth/login.html", &context).unwrap()))
}

/// `/auth/register`
pub async fn register_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = data.read().await;
    let user = get_user_from_token!(jar, data.0);

    if user.is_some() {
        return Err(Redirect::to("/"));
    }

    let lang = get_lang!(jar, data.0);
    let context = initial_context(&data.0.0, lang, &user).await;

    Ok(Html(data.1.render("auth/register.html", &context).unwrap()))
}
