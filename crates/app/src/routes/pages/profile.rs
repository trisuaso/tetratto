use super::{PaginatedQuery, render_error};
use crate::{State, assets::initial_context, get_lang, get_user_from_token};
use axum::{
    Extension,
    extract::{Path, Query},
    response::{Html, IntoResponse},
};
use axum_extra::extract::CookieJar;
use tera::Context;
use tetratto_core::model::{Error, auth::User, communities::Community};

/// `/settings`
pub async fn settings_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = data.read().await;
    let user = match get_user_from_token!(jar, data.0) {
        Some(ua) => ua,
        None => {
            return Err(Html(
                render_error(Error::NotAllowed, &jar, &data, &None).await,
            ));
        }
    };

    let settings = user.settings.clone();
    let tokens = user.tokens.clone();

    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &Some(user)).await;

    context.insert(
        "user_settings_serde",
        &serde_json::to_string(&settings)
            .unwrap()
            .replace("\"", "\\\""),
    );
    context.insert(
        "user_tokens_serde",
        &serde_json::to_string(&tokens)
            .unwrap()
            .replace("\"", "\\\""),
    );

    // return
    Ok(Html(
        data.1.render("profile/settings.html", &context).unwrap(),
    ))
}

pub fn profile_context(
    context: &mut Context,
    profile: &User,
    communities: &Vec<Community>,
    is_self: bool,
    is_following: bool,
) {
    context.insert("profile", &profile);
    context.insert("communities", &communities);
    context.insert("is_self", &is_self);
    context.insert("is_following", &is_following);
}

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
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    // check if we're blocked
    if let Some(ref ua) = user {
        if data
            .0
            .get_userblock_by_initiator_receiver(other_user.id, ua.id)
            .await
            .is_ok()
        {
            return Err(Html(
                render_error(Error::NotAllowed, &jar, &data, &user).await,
            ));
        }
    }

    // fetch data
    let posts = match data
        .0
        .get_posts_by_user(other_user.id, 12, props.page)
        .await
    {
        Ok(p) => match data.0.fill_posts_with_community(p).await {
            Ok(p) => p,
            Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
        },
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    let communities = match data.0.get_memberships_by_owner(other_user.id).await {
        Ok(m) => match data.0.fill_communities(m).await {
            Ok(m) => m,
            Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
        },
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    // init context
    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &user).await;

    let is_self = if let Some(ref ua) = user {
        ua.id == other_user.id
    } else {
        false
    };

    let is_following = if let Some(ref ua) = user {
        data.0
            .get_userfollow_by_initiator_receiver(ua.id, other_user.id)
            .await
            .is_ok()
    } else {
        false
    };

    context.insert("posts", &posts);
    profile_context(
        &mut context,
        &other_user,
        &communities,
        is_self,
        is_following,
    );

    // return
    Ok(Html(data.1.render("profile/posts.html", &context).unwrap()))
}
