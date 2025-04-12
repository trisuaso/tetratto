use super::permissions::FinePermission;
use serde::{Deserialize, Serialize};
use totp_rs::TOTP;
use tetratto_shared::{
    hash::{hash_salted, salt},
    snow::AlmostSnowflake,
    unix_epoch_timestamp,
};

/// `(ip, token, creation timestamp)`
pub type Token = (String, String, usize);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: usize,
    pub created: usize,
    pub username: String,
    pub password: String,
    pub salt: String,
    pub settings: UserSettings,
    pub tokens: Vec<Token>,
    pub permissions: FinePermission,
    pub is_verified: bool,
    pub notification_count: usize,
    pub follower_count: usize,
    pub following_count: usize,
    pub last_seen: usize,
    /// The TOTP secret for this profile. An empty value means the user has TOTP disabled.
    #[serde(default)]
    pub totp: String,
    /// The TOTP recovery codes for this profile.
    #[serde(default)]
    pub recovery_codes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ThemePreference {
    Auto,
    Dark,
    Light,
}

impl Default for ThemePreference {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserSettings {
    #[serde(default)]
    pub policy_consent: bool,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub biography: String,
    #[serde(default)]
    pub warning: String,
    #[serde(default)]
    pub private_profile: bool,
    #[serde(default)]
    pub private_communities: bool,
    /// The theme shown to the user.
    #[serde(default)]
    pub theme_preference: ThemePreference,
    /// The theme used on the user's profile. Setting this to `Auto` will use
    /// the viewing user's `theme_preference` setting.
    #[serde(default)]
    pub profile_theme: ThemePreference,
    #[serde(default)]
    pub private_last_seen: bool,
    #[serde(default)]
    pub theme_hue: String,
    #[serde(default)]
    pub theme_sat: String,
    #[serde(default)]
    pub theme_lit: String,
    /// Page background.
    #[serde(default)]
    pub theme_color_surface: String,
    /// Text on elements with the surface backgrounds.
    #[serde(default)]
    pub theme_color_text: String,
    /// Links on all elements.
    #[serde(default)]
    pub theme_color_text_link: String,
    /// Some cards, buttons, or anything else with a darker background color than the surface.
    #[serde(default)]
    pub theme_color_lowered: String,
    /// Text on elements with the lowered backgrounds.
    #[serde(default)]
    pub theme_color_text_lowered: String,
    /// Borders.
    #[serde(default)]
    pub theme_color_super_lowered: String,
    /// Some cards, buttons, or anything else with a lighter background color than the surface.
    #[serde(default)]
    pub theme_color_raised: String,
    /// Text on elements with the raised backgrounds.
    #[serde(default)]
    pub theme_color_text_raised: String,
    /// Some borders.
    #[serde(default)]
    pub theme_color_super_raised: String,
    /// Primary color; navigation bar, some buttons, etc.
    #[serde(default)]
    pub theme_color_primary: String,
    /// Text on elements with the primary backgrounds.
    #[serde(default)]
    pub theme_color_text_primary: String,
    /// Hover state for primary buttons.
    #[serde(default)]
    pub theme_color_primary_lowered: String,
    /// Secondary color.
    #[serde(default)]
    pub theme_color_secondary: String,
    /// Text on elements with the secondary backgrounds.
    #[serde(default)]
    pub theme_color_text_secondary: String,
    /// Hover state for secondary buttons.
    #[serde(default)]
    pub theme_color_secondary_lowered: String,
    #[serde(default)]
    pub disable_other_themes: bool,
}

impl Default for User {
    fn default() -> Self {
        Self::new("<unknown>".to_string(), String::new())
    }
}

impl User {
    /// Create a new [`User`].
    pub fn new(username: String, password: String) -> Self {
        let salt = salt();
        let password = hash_salted(password, salt.clone());

        Self {
            id: AlmostSnowflake::new(1234567890)
                .to_string()
                .parse::<usize>()
                .unwrap(),
            created: unix_epoch_timestamp() as usize,
            username,
            password,
            salt,
            settings: UserSettings::default(),
            tokens: Vec::new(),
            permissions: FinePermission::DEFAULT,
            is_verified: false,
            notification_count: 0,
            follower_count: 0,
            following_count: 0,
            last_seen: unix_epoch_timestamp() as usize,
            totp: String::new(),
            recovery_codes: Vec::new(),
        }
    }

    /// Deleted user profile.
    pub fn deleted() -> Self {
        Self {
            username: "<deleted>".to_string(),
            id: 0,
            ..Default::default()
        }
    }

    /// Banned user profile.
    pub fn banned() -> Self {
        Self {
            username: "<banned>".to_string(),
            id: 0,
            ..Default::default()
        }
    }

    /// Create a new token
    ///
    /// # Returns
    /// `(unhashed id, token)`
    pub fn create_token(ip: &str) -> (String, Token) {
        let unhashed = tetratto_shared::hash::uuid();
        (
            unhashed.clone(),
            (
                ip.to_string(),
                tetratto_shared::hash::hash(unhashed),
                unix_epoch_timestamp() as usize,
            ),
        )
    }

    /// Check if the given password is correct for the user.
    pub fn check_password(&self, against: String) -> bool {
        self.password == hash_salted(against, self.salt.clone())
    }

    /// Parse user mentions in a given `input`.
    pub fn parse_mentions(input: &str) -> Vec<String> {
        // state
        let mut escape: bool = false;
        let mut at: bool = false;
        let mut buffer: String = String::new();
        let mut out = Vec::new();

        // parse
        for char in input.chars() {
            if (char == '\\') && !escape {
                escape = true;
            }

            if (char == '@') && !escape {
                at = true;
                continue; // don't push @
            }

            if at {
                if (char == ' ') && !escape {
                    // reached space, end @
                    at = false;

                    if !out.contains(&buffer) {
                        out.push(buffer);
                    }

                    buffer = String::new();
                    continue;
                }

                // push mention text
                buffer.push(char);
            }

            escape = false;
        }

        // return
        out
    }

    /// Get a [`TOTP`] from the profile's `totp` secret value.
    pub fn totp(&self, issuer: Option<String>) -> Option<TOTP> {
        if self.totp.is_empty() {
            return None;
        }

        TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            self.totp.as_bytes().to_owned(),
            Some(issuer.unwrap_or("tetratto!".to_string())),
            self.username.clone(),
        )
        .ok()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    pub id: usize,
    pub created: usize,
    pub title: String,
    pub content: String,
    pub owner: usize,
    pub read: bool,
}

impl Notification {
    /// Returns a new [`Notification`].
    pub fn new(title: String, content: String, owner: usize) -> Self {
        Self {
            id: AlmostSnowflake::new(1234567890)
                .to_string()
                .parse::<usize>()
                .unwrap(),
            created: unix_epoch_timestamp() as usize,
            title,
            content,
            owner,
            read: false,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserFollow {
    pub id: usize,
    pub created: usize,
    pub initiator: usize,
    pub receiver: usize,
}

impl UserFollow {
    /// Create a new [`UserFollow`].
    pub fn new(initiator: usize, receiver: usize) -> Self {
        Self {
            id: AlmostSnowflake::new(1234567890)
                .to_string()
                .parse::<usize>()
                .unwrap(),
            created: unix_epoch_timestamp() as usize,
            initiator,
            receiver,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserBlock {
    pub id: usize,
    pub created: usize,
    pub initiator: usize,
    pub receiver: usize,
}

impl UserBlock {
    /// Create a new [`UserBlock`].
    pub fn new(initiator: usize, receiver: usize) -> Self {
        Self {
            id: AlmostSnowflake::new(1234567890)
                .to_string()
                .parse::<usize>()
                .unwrap(),
            created: unix_epoch_timestamp() as usize,
            initiator,
            receiver,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct IpBan {
    pub ip: String,
    pub created: usize,
    pub reason: String,
    pub moderator: usize,
}

impl IpBan {
    /// Create a new [`IpBan`].
    pub fn new(ip: String, moderator: usize, reason: String) -> Self {
        Self {
            ip,
            created: unix_epoch_timestamp() as usize,
            reason,
            moderator,
        }
    }
}
