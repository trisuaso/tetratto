use crate::{State, assets::initial_context, get_lang, get_user_from_token};
use axum::{
    Extension,
    extract::Path,
    response::{Html, IntoResponse},
};
use axum_extra::extract::CookieJar;

/// `/user/{username}`
pub async fn posts_request(
    jar: CookieJar,
    Path(username): Path<String>,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = data.read().await;
    let user = get_user_from_token!(jar, data.0);

    let other_user = match data.0.get_user_by_username(&username).await {
        Ok(ua) => ua,
        Err(e) => return Err(Html(e.to_string())),
    };

    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &user);
    context.insert("profile", &other_user);

    Ok(Html(
        data.1.render("profile/posts.html", &mut context).unwrap(),
    ))
}
