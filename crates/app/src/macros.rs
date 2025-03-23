#[macro_export]
macro_rules! write_template {
    ($html_path:ident->$path:literal($as:expr)) => {
        std::fs::write($html_path.join($path), $as).unwrap();
    };

    ($html_path:ident->$path:literal($as:expr) -d $dir_path:literal) => {
        let dir = $html_path.join($dir_path);
        if !std::fs::exists(&dir).unwrap() {
            std::fs::create_dir(dir).unwrap();
        }

        std::fs::write($html_path.join($path), $as).unwrap();
    };
}

#[macro_export]
macro_rules! create_dir_if_not_exists {
    ($dir_path:expr) => {
        if !std::fs::exists(&$dir_path).unwrap() {
            std::fs::create_dir($dir_path).unwrap();
        }
    };
}

#[macro_export]
macro_rules! get_user_from_token {
    (($jar:ident, $db:expr) <optional>) => {{
        if let Some(token) = $jar.get("__Secure-atto-token") {
            match $db
                .get_user_by_token(&rainbeam_shared::hash::hash(
                    token.to_string().replace("__Secure-atto-token=", ""),
                ))
                .await
            {
                Ok(ua) => Some(ua),
                Err(_) => None,
            }
        } else {
            None
        }
    }};

    ($jar:ident, $db:ident) => {{
        if let Some(token) = $jar.get("__Secure-Atto-Token") {
            match $db.get_user_by_token(token) {
                Ok(ua) => ua,
                Err(_) => return axum::response::Html(crate::data::assets::REDIRECT_TO_AUTH),
            }
        } else {
            None
        }
    }};
}
