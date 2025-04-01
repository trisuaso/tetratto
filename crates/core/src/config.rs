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
    /// If registrations are enabled.
    #[serde(default = "default_security_admin_user")]
    pub admin_user: String,
    /// If registrations are enabled.
    #[serde(default = "default_real_ip_header")]
    pub real_ip_header: String,
}

fn default_security_registration_enabled() -> bool {
    true
}

fn default_security_admin_user() -> String {
    "admin".to_string()
}

fn default_real_ip_header() -> String {
    "CF-Connecting-IP".to_string()
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            registration_enabled: default_security_registration_enabled(),
            admin_user: default_security_admin_user(),
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

impl Default for DirsConfig {
    fn default() -> Self {
        Self {
            templates: default_dir_templates(),
            assets: default_dir_assets(),
            media: default_dir_media(),
            icons: default_dir_icons(),
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
}

fn default_name() -> String {
    "Tetratto".to_string()
}

fn default_description() -> String {
    "🐐 tetratto!".to_string()
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
    ]
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: default_name(),
            description: default_description(),
            color: default_color(),
            port: default_port(),
            banned_hosts: default_banned_hosts(),
            database: default_database(),
            security: default_security(),
            dirs: default_dirs(),
            no_track: default_no_track(),
            banned_usernames: default_banned_usernames(),
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
