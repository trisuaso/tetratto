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
}

fn default_security_registration_enabled() -> bool {
    true
}

fn default_security_admin_user() -> String {
    "admin".to_string()
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            registration_enabled: default_security_registration_enabled(),
            admin_user: default_security_admin_user(),
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
}

fn default_dir_templates() -> String {
    "html".to_string()
}

fn default_dir_assets() -> String {
    "public".to_string()
}

impl Default for DirsConfig {
    fn default() -> Self {
        Self {
            templates: default_dir_templates(),
            assets: default_dir_assets(),
        }
    }
}

/// Configuration file
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    /// The name of the app for templates.
    #[serde(default = "default_name")]
    pub name: String,
    /// The port to serve the server on.
    #[serde(default = "default_port")]
    pub port: u16,
    /// The name of the file to store the SQLite database in.
    #[serde(default = "default_database")]
    pub database: String,
    /// Database security.
    #[serde(default = "default_security")]
    pub security: SecurityConfig,
    /// The locations where different files should be matched.
    #[serde(default = "default_dirs")]
    pub dirs: DirsConfig,
}

fn default_name() -> String {
    "Tetratto".to_string()
}

fn default_port() -> u16 {
    4118
}

fn default_database() -> String {
    "atto.db".to_string()
}

fn default_security() -> SecurityConfig {
    SecurityConfig::default()
}

fn default_dirs() -> DirsConfig {
    DirsConfig::default()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: default_name(),
            port: default_port(),
            database: default_database(),
            security: default_security(),
            dirs: default_dirs(),
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
