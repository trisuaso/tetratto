use crate::{State, assets::initial_context, get_user_from_token};
use axum::{
    Extension,
    response::{Html, IntoResponse},
};
use axum_extra::extract::CookieJar;

/// `/`
pub async fn index_request(jar: CookieJar, Extension(data): Extension<State>) -> impl IntoResponse {
    let data = data.read().await;
    let user = get_user_from_token!((jar, data.0) <optional>);

    let mut context = initial_context(&data.0.0, &user);
    Html(data.1.render("misc/index.html", &mut context).unwrap())
}
