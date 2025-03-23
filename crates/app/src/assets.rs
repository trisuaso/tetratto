use pathbufd::PathBufD;
use tera::Context;
use tetratto_core::{config::Config, model::auth::User};

use crate::write_template;

// images
pub const DEFAULT_AVATAR: &str = include_str!("./public/images/default-avatar.svg");
pub const DEFAULT_BANNER: &str = include_str!("./public/images/default-banner.svg");

// css
pub const STYLE_CSS: &str = include_str!("./public/css/style.css");

// js
pub const ATTO_JS: &str = include_str!("./public/js/atto.js");
pub const LOADER_JS: &str = include_str!("./public/js/loader.js");

// html
pub const ROOT: &str = include_str!("./public/html/root.html");
pub const MACROS: &str = include_str!("./public/html/macros.html");

pub const MISC_INDEX: &str = include_str!("./public/html/misc/index.html");

pub const AUTH_BASE: &str = include_str!("./public/html/auth/base.html");
pub const AUTH_LOGIN: &str = include_str!("./public/html/auth/login.html");
pub const AUTH_REGISTER: &str = include_str!("./public/html/auth/register.html");

// ...

/// Set up public directories.
pub(crate) fn write_assets(html_path: &PathBufD) {
    write_template!(html_path->"root.html"(crate::assets::ROOT));
    write_template!(html_path->"macros.html"(crate::assets::MACROS));

    write_template!(html_path->"misc/index.html"(crate::assets::MISC_INDEX) -d "misc");

    write_template!(html_path->"auth/base.html"(crate::assets::AUTH_BASE) -d "auth");
    write_template!(html_path->"auth/login.html"(crate::assets::AUTH_LOGIN));
    write_template!(html_path->"auth/register.html"(crate::assets::AUTH_REGISTER));
}

/// Create the initial template context.
pub(crate) fn initial_context(config: &Config, user: &Option<User>) -> Context {
    let mut ctx = Context::new();
    ctx.insert("config", &config);
    ctx.insert("user", &user);
    ctx
}
