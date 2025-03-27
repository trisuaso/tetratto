use super::{PaginatedQuery, render_error};
use crate::{State, assets::initial_context, get_lang, get_user_from_token};
use axum::{
    Extension,
    extract::{Path, Query},
    response::{Html, IntoResponse},
};
use axum_extra::extract::CookieJar;

/// `/user/{username}`
pub async fn posts_request(
    jar: CookieJar,
    Path(username): Path<String>,
    Query(props): Query<PaginatedQuery>,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = data.read().await;
    let user = get_user_from_token!(jar, data.0);

    let other_user = match data.0.get_user_by_username(&username).await {
        Ok(ua) => ua,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user))),
    };

    let posts = match data
        .0
        .get_posts_by_user(other_user.id, 12, props.page)
        .await
    {
        Ok(p) => p,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user))),
    };

    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &user);

    context.insert("profile", &other_user);
    context.insert("posts", &posts);

    Ok(Html(
        data.1.render("profile/posts.html", &mut context).unwrap(),
    ))
}
