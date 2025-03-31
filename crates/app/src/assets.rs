use pathbufd::PathBufD;
use regex::Regex;
use std::{
    collections::HashMap,
    fs::{exists, read_to_string, write},
    sync::LazyLock,
};
use tera::Context;
use tetratto_core::{config::Config, model::auth::User};
use tetratto_l10n::LangFile;
use tetratto_shared::hash::salt;
use tokio::sync::RwLock;

use crate::{create_dir_if_not_exists, write_if_track, write_template};

// images
pub const DEFAULT_AVATAR: &str = include_str!("./public/images/default-avatar.svg");
pub const DEFAULT_BANNER: &str = include_str!("./public/images/default-banner.svg");
pub const FAVICON: &str = include_str!("./public/images/favicon.svg");

// css
pub const STYLE_CSS: &str = include_str!("./public/css/style.css");

// js
pub const LOADER_JS: &str = include_str!("./public/js/loader.js");
pub const ATTO_JS: &str = include_str!("./public/js/atto.js");
pub const ME_JS: &str = include_str!("./public/js/me.js");

// html
pub const ROOT: &str = include_str!("./public/html/root.html");
pub const MACROS: &str = include_str!("./public/html/macros.html");
pub const COMPONENTS: &str = include_str!("./public/html/components.html");

pub const MISC_INDEX: &str = include_str!("./public/html/misc/index.html");
pub const MISC_ERROR: &str = include_str!("./public/html/misc/error.html");
pub const MISC_NOTIFICATIONS: &str = include_str!("./public/html/misc/notifications.html");

pub const AUTH_BASE: &str = include_str!("./public/html/auth/base.html");
pub const AUTH_LOGIN: &str = include_str!("./public/html/auth/login.html");
pub const AUTH_REGISTER: &str = include_str!("./public/html/auth/register.html");

pub const PROFILE_BASE: &str = include_str!("./public/html/profile/base.html");
pub const PROFILE_POSTS: &str = include_str!("./public/html/profile/posts.html");
pub const PROFILE_SETTINGS: &str = include_str!("./public/html/profile/settings.html");

pub const COMMUNITIES_LIST: &str = include_str!("./public/html/communities/list.html");
pub const COMMUNITIES_BASE: &str = include_str!("./public/html/communities/base.html");
pub const COMMUNITIES_FEED: &str = include_str!("./public/html/communities/feed.html");
pub const COMMUNITIES_POST: &str = include_str!("./public/html/communities/post.html");
pub const COMMUNITIES_SETTINGS: &str = include_str!("./public/html/communities/settings.html");

pub const TIMELINES_HOME: &str = include_str!("./public/html/timelines/home.html");
pub const TIMELINES_POPULAR: &str = include_str!("./public/html/timelines/popular.html");

// langs
pub const LANG_EN_US: &str = include_str!("./langs/en-US.toml");

// ...

/// A container for all loaded icons.
pub(crate) static ICONS: LazyLock<RwLock<HashMap<String, String>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Pull an icon given its name and insert it into [`ICONS`].
pub(crate) async fn pull_icon(icon: &str, icons_dir: &str) {
    let writer = &mut ICONS.write().await;

    let icon_url = format!(
        "https://raw.githubusercontent.com/lucide-icons/lucide/refs/heads/main/icons/{icon}.svg"
    );

    let file_path = PathBufD::current().extend(&[icons_dir, icon]);

    if exists(&file_path).unwrap() {
        writer.insert(icon.to_string(), read_to_string(&file_path).unwrap());
        return;
    }

    println!("download icon: {icon}");
    let svg = reqwest::get(icon_url).await.unwrap().text().await.unwrap();

    write(&file_path, &svg).unwrap();
    writer.insert(icon.to_string(), svg);
}

/// Read a string and replace all custom blocks with the corresponding correct HTML.
///
/// # Replaces
/// * icons
/// * icons (with class specifier)
/// * l10n text
pub(crate) async fn replace_in_html(input: &str, config: &Config) -> String {
    let mut input = input.to_string();
    input = input.replace("<!-- prettier-ignore -->", "");

    // l10n text
    let text = Regex::new("(\\{\\{)\\s*(text)\\s*\"(.*?)\"\\s*(\\}\\})").unwrap();

    for cap in text.captures_iter(&input.clone()) {
        let replace_with = format!("{{{{ lang[\"{}\"] }}}}", cap.get(3).unwrap().as_str());
        input = input.replace(cap.get(0).unwrap().as_str(), &replace_with);
    }

    // icon (with class)
    let icon_with_class =
        Regex::new("(\\{\\{)\\s*(icon)\\s*(.*?)\\s*c\\((.*?)\\)\\s*(\\}\\})").unwrap();

    for cap in icon_with_class.captures_iter(&input.clone()) {
        let icon = &cap.get(3).unwrap().as_str().replace("\"", "");

        pull_icon(icon, &config.dirs.icons).await;

        let reader = ICONS.read().await;
        let icon_text = reader.get(icon).unwrap().replace(
            "<svg",
            &format!("<svg class=\"icon {}\"", cap.get(4).unwrap().as_str()),
        );

        input = input.replace(cap.get(0).unwrap().as_str(), &icon_text);
    }

    // icon (without class)
    let icon_without_class = Regex::new("(\\{\\{)\\s*(icon)\\s*(.*?)\\s*(\\}\\})").unwrap();

    for cap in icon_without_class.captures_iter(&input.clone()) {
        let icon = &cap.get(3).unwrap().as_str().replace("\"", "");

        pull_icon(icon, &config.dirs.icons).await;

        let reader = ICONS.read().await;
        let icon_text = reader
            .get(icon)
            .unwrap()
            .replace("<svg", "<svg class=\"icon\"");

        input = input.replace(cap.get(0).unwrap().as_str(), &icon_text);
    }

    // return
    input
}

/// Set up public directories.
pub(crate) async fn write_assets(config: &Config) -> PathBufD {
    let html_path = PathBufD::current().join(&config.dirs.templates);

    write_template!(html_path->"root.html"(crate::assets::ROOT) --config=config);
    write_template!(html_path->"macros.html"(crate::assets::MACROS) --config=config);
    write_template!(html_path->"components.html"(crate::assets::COMPONENTS) --config=config);

    write_template!(html_path->"misc/index.html"(crate::assets::MISC_INDEX) -d "misc" --config=config);
    write_template!(html_path->"misc/error.html"(crate::assets::MISC_ERROR) --config=config);
    write_template!(html_path->"misc/notifications.html"(crate::assets::MISC_NOTIFICATIONS) --config=config);

    write_template!(html_path->"auth/base.html"(crate::assets::AUTH_BASE) -d "auth" --config=config);
    write_template!(html_path->"auth/login.html"(crate::assets::AUTH_LOGIN) --config=config);
    write_template!(html_path->"auth/register.html"(crate::assets::AUTH_REGISTER) --config=config);

    write_template!(html_path->"profile/base.html"(crate::assets::PROFILE_BASE) -d "profile" --config=config);
    write_template!(html_path->"profile/posts.html"(crate::assets::PROFILE_POSTS) --config=config);
    write_template!(html_path->"profile/settings.html"(crate::assets::PROFILE_SETTINGS) --config=config);

    write_template!(html_path->"communities/list.html"(crate::assets::COMMUNITIES_LIST) -d "communities" --config=config);
    write_template!(html_path->"communities/base.html"(crate::assets::COMMUNITIES_BASE) --config=config);
    write_template!(html_path->"communities/feed.html"(crate::assets::COMMUNITIES_FEED) --config=config);
    write_template!(html_path->"communities/post.html"(crate::assets::COMMUNITIES_POST) --config=config);
    write_template!(html_path->"communities/settings.html"(crate::assets::COMMUNITIES_SETTINGS) --config=config);

    write_template!(html_path->"timelines/home.html"(crate::assets::TIMELINES_HOME) -d "timelines" --config=config);
    write_template!(html_path->"timelines/popular.html"(crate::assets::TIMELINES_POPULAR) --config=config);

    html_path
}

/// Set up extra directories.
pub(crate) async fn init_dirs(config: &Config) {
    // images
    create_dir_if_not_exists!(&config.dirs.media);
    let images_path = PathBufD::current().extend(&[config.dirs.media.as_str(), "images"]);
    create_dir_if_not_exists!(&images_path);
    create_dir_if_not_exists!(
        &PathBufD::current().extend(&[config.dirs.media.as_str(), "avatars"])
    );
    create_dir_if_not_exists!(
        &PathBufD::current().extend(&[config.dirs.media.as_str(), "community_avatars"])
    );
    create_dir_if_not_exists!(
        &PathBufD::current().extend(&[config.dirs.media.as_str(), "banners"])
    );
    create_dir_if_not_exists!(
        &PathBufD::current().extend(&[config.dirs.media.as_str(), "community_banners"])
    );

    write_if_track!(images_path->"default-avatar.svg"(DEFAULT_AVATAR) --config=config);
    write_if_track!(images_path->"default-banner.svg"(DEFAULT_BANNER) --config=config);
    write_if_track!(images_path->"favicon.svg"(FAVICON) --config=config);

    // icons
    create_dir_if_not_exists!(&PathBufD::current().join(config.dirs.icons.as_str()));

    // langs
    let langs_path = PathBufD::current().join("langs");
    create_dir_if_not_exists!(&langs_path);

    write_template!(langs_path->"en-US.toml"(LANG_EN_US));
}

/// A random ASCII value inserted into the URL of static assets to "break" the cache. Essentially just for cache busting.
pub(crate) static CACHE_BREAKER: LazyLock<String> = LazyLock::new(salt);

/// Create the initial template context.
pub(crate) async fn initial_context(
    config: &Config,
    lang: &LangFile,
    user: &Option<User>,
) -> Context {
    let mut ctx = Context::new();
    ctx.insert("config", &config);
    ctx.insert("user", &user);

    if let Some(ua) = user {
        ctx.insert("is_helper", &ua.permissions.check_helper());
        ctx.insert("is_manager", &ua.permissions.check_manager());
    } else {
        ctx.insert("is_helper", &false);
        ctx.insert("is_manager", &false);
    }

    ctx.insert("lang", &lang.data);
    ctx.insert("random_cache_breaker", &CACHE_BREAKER.clone());
    ctx
}
