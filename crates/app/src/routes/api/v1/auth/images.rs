use axum::{
    Extension, Json,
    body::Body,
    extract::{Path, Query},
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use pathbufd::{PathBufD, pathd};
use serde::Deserialize;
use std::{
    fs::{File, exists},
    io::Read,
};
use tetratto_core::model::{ApiReturn, Error};

use crate::{
    State,
    avif::{Image, save_avif_buffer},
    get_user_from_token,
};

pub fn read_image(path: PathBufD) -> Vec<u8> {
    let mut bytes = Vec::new();

    for byte in File::open(path).unwrap().bytes() {
        bytes.push(byte.unwrap())
    }

    bytes
}

#[derive(Deserialize, PartialEq, Eq)]
pub enum AvatarSelectorType {
    #[serde(alias = "username")]
    Username,
    #[serde(alias = "id")]
    Id,
}

#[derive(Deserialize)]
pub struct AvatarSelectorQuery {
    pub selector_type: AvatarSelectorType,
}

/// Get a profile's avatar image
/// `/api/v1/auth/profile/{id}/avatar`
pub async fn avatar_request(
    Path(selector): Path<String>,
    Extension(data): Extension<State>,
    Query(req): Query<AvatarSelectorQuery>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;

    let user = match {
        if req.selector_type == AvatarSelectorType::Id {
            data.get_user_by_id(selector.parse::<usize>().unwrap())
                .await
        } else {
            data.get_user_by_username(&selector).await
        }
    } {
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
        "avatars",
        &format!("{}.avif", &user.id),
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
    Path(username): Path<String>,
    Extension(data): Extension<State>,
) -> impl IntoResponse {
    let data = &(data.read().await).0;

    let user = match data.get_user_by_username(&username).await {
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
        "banners",
        &format!("{}.avif", &user.id),
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

pub static MAXIUMUM_FILE_SIZE: usize = 8388608;

/// Upload avatar
pub async fn upload_avatar_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    img: Image,
) -> impl IntoResponse {
    // get user from token
    let data = &(data.read().await).0;
    let auth_user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    let path = pathd!("{}/avatars/{}.avif", data.0.dirs.media, &auth_user.id);

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
    Extension(data): Extension<State>,
    img: Image,
) -> impl IntoResponse {
    // get user from token
    let data = &(data.read().await).0;
    let auth_user = match get_user_from_token!(jar, data) {
        Some(ua) => ua,
        None => return Json(Error::NotAllowed.into()),
    };

    let path = pathd!("{}/banners/{}.avif", data.0.dirs.media, &auth_user.id);

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
