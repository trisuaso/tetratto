#[macro_export]
macro_rules! write_template {
    ($into:ident->$path:literal($as:expr)) => {
        std::fs::write($into.join($path), $as).unwrap();
    };

    ($into:ident->$path:literal($as:expr) --config=$config:ident) => {
        std::fs::write(
            $into.join($path),
            $crate::assets::replace_in_html($as, &$config).await,
        )
        .unwrap();
    };

    ($into:ident->$path:literal($as:expr) -d $dir_path:literal) => {
        let dir = $into.join($dir_path);
        if !std::fs::exists(&dir).unwrap() {
            std::fs::create_dir(dir).unwrap();
        }

        std::fs::write($into.join($path), $as).unwrap();
    };

    ($into:ident->$path:literal($as:expr) -d $dir_path:literal --config=$config:ident) => {
        let dir = $into.join($dir_path);
        if !std::fs::exists(&dir).unwrap() {
            std::fs::create_dir(dir).unwrap();
        }

        std::fs::write(
            $into.join($path),
            $crate::assets::replace_in_html($as, &$config).await,
        )
        .unwrap();
    };
}

#[macro_export]
macro_rules! write_if_track {
    ($into:ident->$path:literal($as:expr) --config=$config:ident) => {
        if !$config.no_track.contains(&$path.to_string()) {
            write_template!($into->$path($as));
        }
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
    ($jar:ident, $db:expr) => {{
        if let Some(token) = $jar.get("__Secure-atto-token") {
            match $db
                .get_user_by_token(&tetratto_shared::hash::hash(
                    token.to_string().replace("__Secure-atto-token=", ""),
                ))
                .await
            {
                Ok(ua) => {
                    if ua.permissions.check_banned() {
                        Some(tetratto_core::model::auth::User::banned())
                    } else {
                        Some(ua)
                    }
                }
                Err(_) => None,
            }
        } else {
            None
        }
    }};
}

#[macro_export]
macro_rules! get_lang {
    ($jar:ident, $db:expr) => {{
        if let Some(lang) = $jar.get("__Secure-atto-lang") {
            match $db
                .1
                .get(&lang.to_string().replace("__Secure-atto-lang=", ""))
            {
                Some(lang) => lang,
                None => $db.1.get("com.tetratto.langs:en-US").unwrap(),
            }
        } else {
            $db.1.get("com.tetratto.langs:en-US").unwrap()
        }
    }};
}
