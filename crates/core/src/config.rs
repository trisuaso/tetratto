use pathbufd::PathBufD;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Result;

/// Security configuration.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SecurityConfig {
    /// If registrations are enabled.
    #[serde(default = "default_security_registration_enabled")]
    pub registration_enabled: bool,
    /// The name of the header which will contain the real IP of the connecting user.
    #[serde(default = "default_real_ip_header")]
    pub real_ip_header: String,
}

fn default_security_registration_enabled() -> bool {
    true
}

fn default_real_ip_header() -> String {
    "CF-Connecting-IP".to_string()
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            registration_enabled: default_security_registration_enabled(),
            real_ip_header: default_real_ip_header(),
        }
    }
}

/// Directories configuration.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DirsConfig {
    /// HTML templates directory.
    #[serde(default = "default_dir_templates")]
    pub templates: String,
    /// Static files directory.
    #[serde(default = "default_dir_assets")]
    pub assets: String,
    /// Media (user avatars/banners) files directory.
    #[serde(default = "default_dir_media")]
    pub media: String,
    /// The icons files directory.
    #[serde(default = "default_dir_icons")]
    pub icons: String,
    /// The markdown document files directory.
    #[serde(default = "default_dir_docs")]
    pub docs: String,
}

fn default_dir_templates() -> String {
    "html".to_string()
}

fn default_dir_assets() -> String {
    "public".to_string()
}

fn default_dir_media() -> String {
    "media".to_string()
}

fn default_dir_icons() -> String {
    "icons".to_string()
}

fn default_dir_docs() -> String {
    "docs".to_string()
}

impl Default for DirsConfig {
    fn default() -> Self {
        Self {
            templates: default_dir_templates(),
            assets: default_dir_assets(),
            media: default_dir_media(),
            icons: default_dir_icons(),
            docs: default_dir_docs(),
        }
    }
}

/// Database configuration.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DatabaseConfig {
    pub name: String,
    #[cfg(feature = "postgres")]
    pub url: String,
    #[cfg(feature = "postgres")]
    pub user: String,
    #[cfg(feature = "postgres")]
    pub password: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            name: "atto.db".to_string(),
            #[cfg(feature = "postgres")]
            url: "localhost:5432".to_string(),
            #[cfg(feature = "postgres")]
            user: "postgres".to_string(),
            #[cfg(feature = "postgres")]
            password: "postgres".to_string(),
        }
    }
}

/// Policies config (TOS/privacy)
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PoliciesConfig {
    /// The link to your terms of service page.
    /// This is relative to `/auth/register` on the site.
    ///
    /// If your TOS is an HTML file located in `./public`, you can put
    /// `/public/tos.html` here (or something).
    pub terms_of_service: String,
    /// The link to your privacy policy page.
    /// This is relative to `/auth/register` on the site.
    ///
    /// Same deal as terms of service page.
    pub privacy: String,
}

impl Default for PoliciesConfig {
    fn default() -> Self {
        Self {
            terms_of_service: "/public/tos.html".to_string(),
            privacy: "/public/privacy.html".to_string(),
        }
    }
}

/// Cloudflare Turnstile configuration
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TurnstileConfig {
    pub site_key: String,
    pub secret_key: String,
}

impl Default for TurnstileConfig {
    fn default() -> Self {
        Self {
            site_key: "1x00000000000000000000AA".to_string(), // always passing, visible
            secret_key: "1x0000000000000000000000000000000AA".to_string(), // always passing
        }
    }
}

/// Configuration file
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    /// The name of the app.
    #[serde(default = "default_name")]
    pub name: String,
    /// The description of the app.
    #[serde(default = "default_description")]
    pub description: String,
    /// The theme color of the app.
    #[serde(default = "default_color")]
    pub color: String,
    /// The port to serve the server on.
    #[serde(default = "default_port")]
    pub port: u16,
    /// A list of hosts which cannot be proxied through the image proxy.
    ///
    /// They will return the default banner image instead of proxying.
    ///
    /// It is recommended to put the host of your own public server in this list in
    /// order to prevent a way too easy DOS.
    #[serde(default = "default_banned_hosts")]
    pub banned_hosts: Vec<String>,
    /// The main public host of the server. **Not** used to check against banned hosts,
    /// so this host should be included in there as well.
    #[serde(default = "default_host")]
    pub host: String,
    /// Database security.
    #[serde(default = "default_security")]
    pub security: SecurityConfig,
    /// The locations where different files should be matched.
    #[serde(default = "default_dirs")]
    pub dirs: DirsConfig,
    /// Database configuration.
    #[serde(default = "default_database")]
    pub database: DatabaseConfig,
    /// A list of files (just their name, no full path) which are NOT updated to match the
    /// version built with the server binary.
    #[serde(default = "default_no_track")]
    pub no_track: Vec<String>,
    /// A list of usernames which cannot be used. This also includes community names.
    #[serde(default = "default_banned_usernames")]
    pub banned_usernames: Vec<String>,
    /// Configuration for your site's policies (terms of service, privacy).
    #[serde(default = "default_policies")]
    pub policies: PoliciesConfig,
    /// Configuration for Cloudflare Turnstile.
    #[serde(default = "default_turnstile")]
    pub turnstile: TurnstileConfig,
    /// The ID of the "town square" community. This community is required to allow
    /// people to post from their profiles.
    ///
    /// This community **must** have open write access.
    #[serde(default)]
    pub town_square: usize,
}

fn default_name() -> String {
    "Tetratto".to_string()
}

fn default_description() -> String {
    "🐇 tetratto!".to_string()
}

fn default_color() -> String {
    "#c9b1bc".to_string()
}
fn default_port() -> u16 {
    4118
}

fn default_banned_hosts() -> Vec<String> {
    Vec::new()
}

fn default_host() -> String {
    String::new()
}

fn default_security() -> SecurityConfig {
    SecurityConfig::default()
}

fn default_dirs() -> DirsConfig {
    DirsConfig::default()
}

fn default_database() -> DatabaseConfig {
    DatabaseConfig::default()
}

fn default_no_track() -> Vec<String> {
    Vec::new()
}

fn default_banned_usernames() -> Vec<String> {
    vec![
        "admin".to_string(),
        "owner".to_string(),
        "moderator".to_string(),
        "api".to_string(),
        "communities".to_string(),
        "notifs".to_string(),
        "notification".to_string(),
        "post".to_string(),
        "void".to_string(),
        "anonymous".to_string(),
    ]
}

fn default_policies() -> PoliciesConfig {
    PoliciesConfig::default()
}

fn default_turnstile() -> TurnstileConfig {
    TurnstileConfig::default()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: default_name(),
            description: default_description(),
            color: default_color(),
            port: default_port(),
            banned_hosts: default_banned_hosts(),
            host: default_host(),
            database: default_database(),
            security: default_security(),
            dirs: default_dirs(),
            no_track: default_no_track(),
            banned_usernames: default_banned_usernames(),
            policies: default_policies(),
            turnstile: default_turnstile(),
            town_square: 0,
        }
    }
}

impl Config {
    /// Read configuration file into [`Config`]
    pub fn read(contents: String) -> Self {
        toml::from_str::<Self>(&contents).unwrap()
    }

    /// Pull configuration file
    pub fn get_config() -> Self {
        let path = PathBufD::current().join("tetratto.toml");

        match fs::read_to_string(&path) {
            Ok(c) => Config::read(c),
            Err(_) => {
                Self::update_config(Self::default()).expect("failed to write default config");
                Self::default()
            }
        }
    }

    /// Update configuration file
    pub fn update_config(contents: Self) -> Result<()> {
        let c = fs::canonicalize(".").unwrap();
        let here = c.to_str().unwrap();

        fs::write(
            format!("{here}/tetratto.toml"),
            toml::to_string_pretty::<Self>(&contents).unwrap(),
        )
    }
}
