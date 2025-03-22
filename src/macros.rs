#[macro_export]
macro_rules! write_template {
    ($atto_dir:ident->$path:literal($as:expr)) => {
        std::fs::write($atto_dir.join($path), $as).unwrap();
    };

    ($atto_dir:ident->$path:literal($as:expr) -d $dir_path:literal) => {
        let dir = $atto_dir.join($dir_path);
        if !std::fs::exists(&dir).unwrap() {
            std::fs::create_dir(dir).unwrap();
        }

        std::fs::write($atto_dir.join($path), $as).unwrap();
    };
}

#[macro_export]
macro_rules! get_user_from_token {
    (($jar:ident, $db:ident) <optional>) => {{
        if let Some(token) = $jar.get("__Secure-Atto-Token") {
            match $db.get_user_by_token(&token.to_string()).await {
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
