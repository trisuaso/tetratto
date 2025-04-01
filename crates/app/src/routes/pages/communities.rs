use super::{PaginatedQuery, render_error};
use crate::{State, assets::initial_context, get_lang, get_user_from_token};
use axum::{
    Extension,
    extract::{Path, Query},
    response::{Html, IntoResponse},
};
use axum_extra::extract::CookieJar;
use tera::Context;
use tetratto_core::model::{
    Error,
    auth::User,
    communities::{Community, CommunityReadAccess},
    communities_permissions::CommunityPermission,
};

macro_rules! check_permissions {
    ($community:ident, $jar:ident, $data:ident, $user:ident) => {{
        let mut is_member: bool = false;

        if let Some(ref ua) = $user {
            if let Ok(membership) = $data
                .0
                .get_membership_by_owner_community(ua.id, $community.id)
                .await
            {
                if membership.role.check_banned() {
                    return Err(Html(
                        render_error(Error::NotAllowed, &$jar, &$data, &$user).await,
                    ));
                } else if membership.role.check_member() {
                    is_member = true;
                }
            }
        }

        match $community.read_access {
            CommunityReadAccess::Joined => {
                if !is_member {
                    false
                } else {
                    true
                }
            }
            _ => true,
        }
    }};
}

macro_rules! community_context_bools {
    ($data:ident, $user:ident, $community:ident) => {{
        let membership = if let Some(ref ua) = $user {
            match $data
                .0
                .get_membership_by_owner_community(ua.id, $community.id)
                .await
            {
                Ok(m) => Some(m),
                Err(_) => None,
            }
        } else {
            None
        };

        let is_owner = if let Some(ref ua) = $user {
            ua.id == $community.owner
        } else {
            false
        };

        let is_joined = if let Some(ref membership) = membership {
            membership.role.check_member()
        } else {
            false
        };

        let is_pending = if let Some(ref membership) = membership {
            membership.role.check(CommunityPermission::REQUESTED)
        } else {
            false
        };

        let can_post = if let Some(ref ua) = $user {
            $data.0.check_can_post(&$community, ua.id).await
        } else {
            false
        };

        (is_owner, is_joined, is_pending, can_post)
    }};
}

/// `/communities`
pub async fn list_request(jar: CookieJar, Extension(data): Extension<State>) -> impl IntoResponse {
    let data = data.read().await;
    let user = match get_user_from_token!(jar, data.0) {
        Some(ua) => ua,
        None => {
            return Err(Html(
                render_error(Error::NotAllowed, &jar, &data, &None).await,
            ));
        }
    };

    let list = match data.0.get_memberships_by_owner(user.id).await {
        Ok(p) => p,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &Some(user)).await)),
    };

    let mut communities: Vec<Community> = Vec::new();
    for membership in &list {
        match data.0.get_community_by_id(membership.community).await {
            Ok(c) => communities.push(c),
            Err(e) => return Err(Html(render_error(e, &jar, &data, &None).await)),
        }
    }

    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &Some(user)).await;
    context.insert("list", &communities);

    // return
    Ok(Html(
        data.1.render("communities/list.html", &context).unwrap(),
    ))
}

pub fn community_context(
    context: &mut Context,
    community: &Community,
    is_owner: bool,
    is_joined: bool,
    is_pending: bool,
    can_post: bool,
    can_read: bool,
) {
    context.insert("community", &community);
    context.insert("is_owner", &is_owner);
    context.insert("is_joined", &is_joined);
    context.insert("is_pending", &is_pending);
    context.insert("can_post", &can_post);
    context.insert("can_read", &can_read);
}

/// `/community/{title}`
pub async fn feed_request(
    jar: CookieJar,
    Path(title): Path<String>,
    Query(props): Query<PaginatedQuery>,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = data.read().await;
    let user = get_user_from_token!(jar, data.0);

    let community = match data.0.get_community_by_title(&title.to_lowercase()).await {
        Ok(ua) => ua,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    if community.id == 0 {
        // don't show page for void community
        return Err(Html(
            render_error(
                Error::GeneralNotFound("community".to_string()),
                &jar,
                &data,
                &user,
            )
            .await,
        ));
    }

    // check permissions
    let can_read = check_permissions!(community, jar, data, user);

    // ...
    let feed = match data
        .0
        .get_posts_by_community(community.id, 12, props.page)
        .await
    {
        Ok(p) => match data.0.fill_posts(p).await {
            Ok(p) => p,
            Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
        },
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    // init context
    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &user).await;

    let (is_owner, is_joined, is_pending, can_post) =
        community_context_bools!(data, user, community);

    context.insert("feed", &feed);
    community_context(
        &mut context,
        &community,
        is_owner,
        is_joined,
        is_pending,
        can_post,
        can_read,
    );

    // return
    Ok(Html(
        data.1.render("communities/feed.html", &context).unwrap(),
    ))
}

/// `/community/{title}/manage`
pub async fn settings_request(
    jar: CookieJar,
    Path(title): Path<String>,
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

    let community = match data.0.get_community_by_title(&title.to_lowercase()).await {
        Ok(ua) => ua,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &Some(user)).await)),
    };

    if user.id != community.owner {
        return Err(Html(
            render_error(Error::NotAllowed, &jar, &data, &None).await,
        ));
    }

    // init context
    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &Some(user)).await;

    context.insert("community", &community);
    context.insert(
        "community_context_serde",
        &serde_json::to_string(&community.context)
            .unwrap()
            .replace("\"", "\\\""),
    );

    // return
    Ok(Html(
        data.1
            .render("communities/settings.html", &context)
            .unwrap(),
    ))
}

/// `/post/{id}`
pub async fn post_request(
    jar: CookieJar,
    Path(id): Path<usize>,
    Query(props): Query<PaginatedQuery>,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = data.read().await;
    let user = get_user_from_token!(jar, data.0);

    let post = match data.0.get_post_by_id(id).await {
        Ok(ua) => ua,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    let community = match data.0.get_community_by_id(post.community).await {
        Ok(ua) => ua,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    // check permissions
    let can_read = check_permissions!(community, jar, data, user);

    // ...
    let feed = match data.0.get_post_comments(post.id, 12, props.page).await {
        Ok(p) => match data.0.fill_posts(p).await {
            Ok(p) => p,
            Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
        },
        Err(e) => return Err(Html(render_error(e, &jar, &data, &user).await)),
    };

    // init context
    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &user).await;

    let (is_owner, is_joined, is_pending, can_post) =
        community_context_bools!(data, user, community);

    context.insert("post", &post);
    context.insert("replies", &feed);
    context.insert(
        "owner",
        &data
            .0
            .get_user_by_id(post.owner)
            .await
            .unwrap_or(User::deleted()),
    );
    community_context(
        &mut context,
        &community,
        is_owner,
        is_joined,
        is_pending,
        can_post,
        can_read,
    );

    // return
    Ok(Html(
        data.1.render("communities/post.html", &context).unwrap(),
    ))
}
