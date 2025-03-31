use axum::{Extension, Json, body::Body, extract::Path, response::IntoResponse};
use axum_extra::extract::CookieJar;
use pathbufd::{PathBufD, pathd};
use std::fs::exists;
use tetratto_core::model::{ApiReturn, Error, permissions::FinePermission};

use crate::{
    State,
    avif::{Image, save_avif_buffer},
    get_user_from_token,
    routes::api::v1::auth::images::{MAXIUMUM_FILE_SIZE, read_image},
};

/// Get a community's avatar image
/// `/api/v1/communities/{id}/avatar`
pub async fn avatar_request(
    Path(id): Path<usize>,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;

    let community = match data.get_community_by_id(id).await {
        Ok(ua) => ua,
        Err(_) => {
            return (
                [("Content-Type", "image/svg+xml")],
                Body::from(read_image(PathBufD::current().extend(&[
                    data.0.dirs.media.as_str(),
                    "images",
                    "default-avatar.svg",
                ]))),
            );
        }
    };

    let path = PathBufD::current().extend(&[
        data.0.dirs.media.as_str(),
        "community_avatars",
        &format!("{}.avif", &community.id),
    ]);

    if !exists(&path).unwrap() {
        return (
            [("Content-Type", "image/svg+xml")],
            Body::from(read_image(PathBufD::current().extend(&[
                data.0.dirs.media.as_str(),
                "images",
                "default-avatar.svg",
            ]))),
        );
    }

    (
        [("Content-Type", "image/avif")],
        Body::from(read_image(path)),
    )
}

/// Get a profile's banner image
/// `/api/v1/auth/profile/{id}/banner`
pub async fn banner_request(
    Path(id): Path<usize>,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;

    let community = match data.get_community_by_id(id).await {
        Ok(ua) => ua,
        Err(_) => {
            return (
                [("Content-Type", "image/svg+xml")],
                Body::from(read_image(PathBufD::current().extend(&[
                    data.0.dirs.media.as_str(),
                    "images",
                    "default-banner.svg",
                ]))),
            );
        }
    };

    let path = PathBufD::current().extend(&[
        data.0.dirs.media.as_str(),
        "community_banners",
        &format!("{}.avif", &community.id),
    ]);

    if !exists(&path).unwrap() {
        return (
            [("Content-Type", "image/svg+xml")],
            Body::from(read_image(PathBufD::current().extend(&[
                data.0.dirs.media.as_str(),
                "images",
                "default-banner.svg",
            ]))),
        );
    }

    (
        [("Content-Type", "image/avif")],
        Body::from(read_image(path)),
    )
}

/// Upload avatar
pub async fn upload_avatar_request(
    jar: CookieJar,
    Path(id): Path<usize>,
    Extension(data): Extension<State>,
    img: Image,
) -> impl IntoResponse {
    // get user from token
    let data = &(data.read().await).0;
    let auth_user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    let community = match data.get_community_by_id(id).await {
        Ok(c) => c,
        Err(e) => return Json(e.into()),
    };

    if auth_user.id != community.owner && !auth_user
            .permissions
            .check(FinePermission::MANAGE_COMMUNITIES) {
        return Json(Error::NotAllowed.into());
    }

    let path = pathd!(
        "{}/community_avatars/{}.avif",
        data.0.dirs.media,
        &community.id
    );

    // check file size
    if img.0.len() > MAXIUMUM_FILE_SIZE {
        return Json(Error::DataTooLong("image".to_string()).into());
    }

    // upload image
    let mut bytes = Vec::new();

    for byte in img.0 {
        bytes.push(byte);
    }

    match save_avif_buffer(&path, bytes) {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Avatar uploaded. It might take a bit to update".to_string(),
            payload: (),
        }),
        Err(e) => Json(Error::MiscError(e.to_string()).into()),
    }
}

/// Upload banner
pub async fn upload_banner_request(
    jar: CookieJar,
    Path(id): Path<usize>,
    Extension(data): Extension<State>,
    img: Image,
) -> impl IntoResponse {
    // get user from token
    let data = &(data.read().await).0;
    let auth_user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    let community = match data.get_community_by_id(id).await {
        Ok(c) => c,
        Err(e) => return Json(e.into()),
    };

    if auth_user.id != community.owner && !auth_user
            .permissions
            .check(FinePermission::MANAGE_COMMUNITIES) {
        return Json(Error::NotAllowed.into());
    }

    let path = pathd!(
        "{}/community_banners/{}.avif",
        data.0.dirs.media,
        &community.id
    );

    // check file size
    if img.0.len() > MAXIUMUM_FILE_SIZE {
        return Json(Error::DataTooLong("image".to_string()).into());
    }

    // upload image
    let mut bytes = Vec::new();

    for byte in img.0 {
        bytes.push(byte);
    }

    match save_avif_buffer(&path, bytes) {
        Ok(_) => Json(ApiReturn {
            ok: true,
            message: "Banner uploaded. It might take a bit to update".to_string(),
            payload: (),
        }),
        Err(e) => Json(Error::MiscError(e.to_string()).into()),
    }
}
