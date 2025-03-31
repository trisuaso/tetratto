use axum::{
    Extension, Json,
    extract::Path,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::CookieJar;
use tetratto_core::model::{
    ApiReturn, Error,
    auth::Notification,
    communities::{Community, CommunityMembership},
    communities_permissions::CommunityPermission,
};

use crate::{
    State, get_user_from_token,
    routes::api::v1::{
        CreateCommunity, UpdateCommunityContext, UpdateCommunityReadAccess, UpdateCommunityTitle,
        UpdateCommunityWriteAccess, UpdateMembershipRole,
    },
};

pub async fn redirect_from_id(
    Extension(data): Extension<State>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match (data.read().await)
        .0
        .get_community_by_id(match id.parse::<usize>() {
            Ok(id) => id,
            Err(_) => return Redirect::to("/"),
        })
        .await
    {
        Ok(c) => Redirect::to(&format!("/community/{}", c.title)),
        Err(_) => Redirect::to("/"),
    }
}

pub async fn create_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Json(req): Json<CreateCommunity>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data
        .create_community(Community::new(req.title, user.id))
        .await
    {
        Ok(id) => Json(ApiReturn {
            ok: true,
            message: "Community created".to_string(),
            payload: Some(id.to_string()),
        }),
        Err(e) => Json(e.into()),
    }
}

pub async fn delete_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path(id): Path<usize>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data.delete_community(id, user).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Community deleted".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
    }
}

pub async fn update_title_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path(id): Path<usize>,
    Json(req): Json<UpdateCommunityTitle>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data.update_community_title(id, user, req.title).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Community updated".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
    }
}

pub async fn update_context_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path(id): Path<usize>,
    Json(req): Json<UpdateCommunityContext>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data.update_community_context(id, user, req.context).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Community updated".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
    }
}

pub async fn update_read_access_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path(id): Path<usize>,
    Json(req): Json<UpdateCommunityReadAccess>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data
        .update_community_read_access(id, user, req.access)
        .await
    {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Community updated".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
    }
}

pub async fn update_write_access_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path(id): Path<usize>,
    Json(req): Json<UpdateCommunityWriteAccess>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data
        .update_community_write_access(id, user, req.access)
        .await
    {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Community updated".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
    }
}

pub async fn get_membership(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path((cid, uid)): Path<(usize, usize)>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    let community = match data.get_community_by_id(cid).await {
        Ok(c) => c,
        Err(e) => return Json(e.into()),
    };

    if user.id != community.owner {
        // only the owner can select community memberships
        return Json(Error::NotAllowed.into());
    }

    match data.get_membership_by_owner_community(uid, cid).await {
        Ok(m) => Json(ApiReturn {
            ok: true,
            message: "Membership exists".to_string(),
            payload: Some(m),
        }),
        Err(e) => Json(e.into()),
    }
}

pub async fn create_membership(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path(id): Path<usize>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    match data
        .create_membership(CommunityMembership::new(
            user.id,
            id,
            CommunityPermission::default(),
        ))
        .await
    {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Community joined".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
    }
}

pub async fn delete_membership(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path((cid, uid)): Path<(usize, usize)>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    let membership = match data.get_membership_by_owner_community(uid, cid).await {
        Ok(c) => c,
        Err(e) => return Json(e.into()),
    };

    match data.delete_membership(membership.id, user).await {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Membership deleted".to_string(),
            payload: (),
        }),
        Err(e) => Json(e.into()),
    }
}

pub async fn update_membership_role(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path((cid, uid)): Path<(usize, usize)>,
    Json(req): Json<UpdateMembershipRole>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;
    let user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    let membership = match data.get_membership_by_owner_community(uid, cid).await {
        Ok(c) => c,
        Err(e) => return Json(e.into()),
    };

    let community = match data.get_community_by_id(membership.community).await {
        Ok(c) => c,
        Err(e) => return Json(e.into()),
    };

    if membership.owner == community.owner {
        return Json(Error::MiscError("Cannot update community owner's role".to_string()).into());
    }

    if user.id != community.owner {
        return Json(Error::NotAllowed.into());
    }

    match data.update_membership_role(membership.id, req.role).await {
        Ok(_) => {
            // check if the user was just banned/unbanned (and send notifs)
            if (req.role & CommunityPermission::BANNED) == CommunityPermission::BANNED {
                // user was banned
                if let Err(e) = data
                    .create_notification(Notification::new(
                        "You have been banned from a community.".to_string(),
                        format!(
                            "You have been banned from [{}](/community/{}).",
                            community.title, community.title
                        ),
                        membership.owner,
                    ))
                    .await
                {
                    return Json(e.into());
                };

                if let Err(e) = data.decr_community_member_count(community.id).await {
                    // banned members do not count towards member count
                    return Json(e.into());
                }
            } else if (membership.role & CommunityPermission::BANNED) == CommunityPermission::BANNED
            {
                // user was unbanned
                if let Err(e) = data
                    .create_notification(Notification::new(
                        "You have been unbanned from a community.".to_string(),
                        format!(
                            "You have been unbanned from [{}](/community/{}).",
                            community.title, community.title
                        ),
                        membership.owner,
                    ))
                    .await
                {
                    return Json(e.into());
                };

                if let Err(e) = data.incr_community_member_count(community.id).await {
                    return Json(e.into());
                }
            }

            Json(ApiReturn {
                ok: true,
                message: "Membership updated".to_string(),
                payload: (),
            })
        }
        Err(e) => Json(e.into()),
    }
}
