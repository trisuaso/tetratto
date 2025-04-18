use super::*;
use crate::cache::Cache;
use crate::model::moderation::AuditLogEntry;
use crate::model::{
    Error, Result,
    auth::{Token, User, UserSettings},
    permissions::FinePermission,
};
use crate::{auto_method, execute, get, query_row, params};
use pathbufd::PathBufD;
use std::fs::{exists, remove_file};
use tetratto_shared::hash::{hash_salted, salt};
use tetratto_shared::unix_epoch_timestamp;

#[cfg(feature = "sqlite")]
use rusqlite::Row;

#[cfg(feature = "postgres")]
use tokio_postgres::Row;

impl DataManager {
    /// Get a [`User`] from an SQL row.
    pub(crate) fn get_user_from_row(
        #[cfg(feature = "sqlite")] x: &Row<'_>,
        #[cfg(feature = "postgres")] x: &Row,
    ) -> User {
        User {
            id: get!(x->0(i64)) as usize,
            created: get!(x->1(i64)) as usize,
            username: get!(x->2(String)),
            password: get!(x->3(String)),
            salt: get!(x->4(String)),
            settings: serde_json::from_str(&get!(x->5(String)).to_string()).unwrap(),
            tokens: serde_json::from_str(&get!(x->6(String)).to_string()).unwrap(),
            permissions: FinePermission::from_bits(get!(x->7(i32)) as u32).unwrap(),
            is_verified: get!(x->8(i32)) as i8 == 1,
            notification_count: get!(x->9(i32)) as usize,
            follower_count: get!(x->10(i32)) as usize,
            following_count: get!(x->11(i32)) as usize,
            last_seen: get!(x->12(i64)) as usize,
            totp: get!(x->13(String)),
            recovery_codes: serde_json::from_str(&get!(x->14(String)).to_string()).unwrap(),
            post_count: get!(x->15(i32)) as usize,
            request_count: get!(x->16(i32)) as usize,
        }
    }

    auto_method!(get_user_by_id(usize as i64)@get_user_from_row -> "SELECT * FROM users WHERE id = $1" --name="user" --returns=User --cache-key-tmpl="atto.user:{}");
    auto_method!(get_user_by_username(&str)@get_user_from_row -> "SELECT * FROM users WHERE username = $1" --name="user" --returns=User --cache-key-tmpl="atto.user:{}");

    /// Get a user given just their ID. Returns the void user if the user doesn't exist.
    ///
    /// # Arguments
    /// * `id` - the ID of the user
    pub async fn get_user_by_id_with_void(&self, id: usize) -> Result<User> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_row!(
            &conn,
            "SELECT * FROM users WHERE id = $1",
            &[&(id as i64)],
            |x| Ok(Self::get_user_from_row(x))
        );

        if res.is_err() {
            return Ok(User::deleted());
            // return Err(Error::UserNotFound);
        }

        Ok(res.unwrap())
    }

    /// Get a user given just their auth token.
    ///
    /// # Arguments
    /// * `token` - the token of the user
    pub async fn get_user_by_token(&self, token: &str) -> Result<User> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_row!(
            &conn,
            "SELECT * FROM users WHERE tokens LIKE $1",
            &[&format!("%\"{token}\"%")],
            |x| Ok(Self::get_user_from_row(x))
        );

        if res.is_err() {
            return Err(Error::UserNotFound);
        }

        Ok(res.unwrap())
    }

    /// Create a new user in the database.
    ///
    /// # Arguments
    /// * `data` - a mock [`User`] object to insert
    pub async fn create_user(&self, mut data: User) -> Result<()> {
        if !self.0.security.registration_enabled {
            return Err(Error::RegistrationDisabled);
        }

        data.username = data.username.to_lowercase();

        // check values
        if data.username.len() < 2 {
            return Err(Error::DataTooShort("username".to_string()));
        } else if data.username.len() > 32 {
            return Err(Error::DataTooLong("username".to_string()));
        }

        if data.password.len() < 6 {
            return Err(Error::DataTooShort("password".to_string()));
        }

        if self.0.banned_usernames.contains(&data.username) {
            return Err(Error::MiscError("This username cannot be used".to_string()));
        }

        // make sure username isn't taken
        if self.get_user_by_username(&data.username).await.is_ok() {
            return Err(Error::UsernameInUse);
        }

        // ...
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "INSERT INTO users VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)",
            params![
                &(data.id as i64),
                &(data.created as i64),
                &data.username.to_lowercase(),
                &data.password,
                &data.salt,
                &serde_json::to_string(&data.settings).unwrap(),
                &serde_json::to_string(&data.tokens).unwrap(),
                &(FinePermission::DEFAULT.bits() as i32),
                &(if data.is_verified { 1_i32 } else { 0_i32 }),
                &0_i32,
                &0_i32,
                &0_i32,
                &(data.last_seen as i64),
                &String::new(),
                &"[]",
                &0_i32,
                &0_i32
            ]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        Ok(())
    }

    /// Delete an existing user in the database.
    ///
    /// # Arguments
    /// * `id` - the ID of the user
    /// * `password` - the current password of the user
    /// * `force` - if we should delete even if the given password is incorrect
    pub async fn delete_user(&self, id: usize, password: &str, force: bool) -> Result<()> {
        let user = self.get_user_by_id(id).await?;

        if (hash_salted(password.to_string(), user.salt.clone()) != user.password) && !force {
            return Err(Error::IncorrectPassword);
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(&conn, "DELETE FROM users WHERE id = $1", &[&(id as i64)]);

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.cache_clear_user(&user).await;

        // delete communities
        let res = execute!(
            &conn,
            "DELETE FROM communities WHERE owner = $1",
            &[&(id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // delete memberships
        // member counts will remain the same... but that should probably be changed
        let res = execute!(
            &conn,
            "DELETE FROM memberships WHERE owner = $1",
            &[&(id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // delete notifications
        let res = execute!(
            &conn,
            "DELETE FROM notifications WHERE owner = $1",
            &[&(id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // delete requests
        let res = execute!(
            &conn,
            "DELETE FROM requests WHERE owner = $1",
            &[&(id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // delete warnings
        let res = execute!(
            &conn,
            "DELETE FROM user_warnings WHERE receiver = $1",
            &[&(id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // delete blocks
        let res = execute!(
            &conn,
            "DELETE FROM userblocks WHERE initiator = $1 OR receiver = $1",
            &[&(id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        let res = execute!(
            &conn,
            "DELETE FROM ipblocks WHERE initiator = $1",
            &[&(id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // delete reactions
        // reactions counts will remain the same :)
        let res = execute!(
            &conn,
            "DELETE FROM reactions WHERE owner = $1",
            &[&(id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // delete posts
        let res = execute!(&conn, "DELETE FROM posts WHERE owner = $1", &[&(id as i64)]);

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // delete user follows... individually since it requires updating user counts
        for follow in self.get_userfollows_by_receiver_all(id).await? {
            self.delete_userfollow(follow.id, &user, true).await?;
        }

        for follow in self.get_userfollows_by_initiator_all(id).await? {
            self.delete_userfollow(follow.id, &user, true).await?;
        }

        // remove images
        let avatar = PathBufD::current().extend(&[
            self.0.dirs.media.as_str(),
            "avatars",
            &format!("{}.avif", &(user.id as i64)),
        ]);

        let banner = PathBufD::current().extend(&[
            self.0.dirs.media.as_str(),
            "banners",
            &format!("{}.avif", &(user.id as i64)),
        ]);

        if exists(&avatar).unwrap() {
            remove_file(avatar).unwrap();
        }

        if exists(&banner).unwrap() {
            remove_file(banner).unwrap();
        }

        // ...
        Ok(())
    }

    pub async fn update_user_verified_status(&self, id: usize, x: bool, user: User) -> Result<()> {
        if !user.permissions.check(FinePermission::MANAGE_VERIFIED) {
            return Err(Error::NotAllowed);
        }

        let other_user = self.get_user_by_id(id).await?;

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "UPDATE users SET verified = $1 WHERE id = $2",
            params![&{ if x { 1 } else { 0 } }, &(id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.cache_clear_user(&other_user).await;

        // create audit log entry
        self.create_audit_log_entry(AuditLogEntry::new(
            user.id,
            format!(
                "invoked `update_user_verified_status` with x value `{}` and y value `{}`",
                other_user.id, x
            ),
        ))
        .await?;

        // ...
        Ok(())
    }

    pub async fn update_user_password(
        &self,
        id: usize,
        from: String,
        to: String,
        user: User,
        force: bool,
    ) -> Result<()> {
        // verify password
        if (hash_salted(from.clone(), user.salt.clone()) != user.password) && !force {
            return Err(Error::MiscError("Password does not match".to_string()));
        }

        // ...
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let new_salt = salt();
        let new_password = hash_salted(to, new_salt.clone());
        let res = execute!(
            &conn,
            "UPDATE users SET password = $1, salt = $2 WHERE id = $3",
            params![&new_password.as_str(), &new_salt.as_str(), &(id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.cache_clear_user(&user).await;

        Ok(())
    }

    pub async fn update_user_username(&self, id: usize, to: String, user: User) -> Result<()> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "UPDATE users SET username = $1 WHERE id = $2",
            params![&to.as_str(), &(id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.cache_clear_user(&user).await;

        Ok(())
    }

    pub async fn update_user_role(
        &self,
        id: usize,
        role: FinePermission,
        user: User,
    ) -> Result<()> {
        // check permission
        if !user.permissions.check(FinePermission::MANAGE_USERS) {
            return Err(Error::NotAllowed);
        }

        let other_user = self.get_user_by_id(id).await?;

        if other_user.permissions.check_manager() && !user.permissions.check_admin() {
            return Err(Error::MiscError(
                "Cannot manage the role of other managers".to_string(),
            ));
        }

        if other_user.permissions == user.permissions {
            return Err(Error::MiscError(
                "Cannot manage users of equal level to you".to_string(),
            ));
        }

        // ...
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "UPDATE users SET permissions = $1 WHERE id = $2",
            params![&(role.bits() as i32), &(id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.cache_clear_user(&other_user).await;

        // create audit log entry
        self.create_audit_log_entry(AuditLogEntry::new(
            user.id,
            format!(
                "invoked `update_user_role` with x value `{}` and y value `{}`",
                other_user.id,
                role.bits()
            ),
        ))
        .await?;

        // ...
        Ok(())
    }

    pub async fn seen_user(&self, user: &User) -> Result<()> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "UPDATE users SET last_seen = $1 WHERE id = $2",
            params![&(unix_epoch_timestamp() as i64), &(user.id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.cache_clear_user(user).await;

        Ok(())
    }

    /// Validate a given TOTP code for the given profile.
    pub fn check_totp(&self, ua: &User, code: &str) -> bool {
        let totp = ua.totp(Some(
            self.0
                .host
                .replace("http://", "")
                .replace("https://", "")
                .replace(":", "_"),
        ));

        if let Some(totp) = totp {
            return !code.is_empty()
                && (totp.check_current(code).unwrap()
                    | ua.recovery_codes.contains(&code.to_string()));
        }

        true
    }

    /// Generate 8 random recovery codes for TOTP.
    pub fn generate_totp_recovery_codes() -> Vec<String> {
        let mut out: Vec<String> = Vec::new();

        for _ in 0..9 {
            out.push(salt())
        }

        out
    }

    /// Update the profile's TOTP secret.
    ///
    /// # Arguments
    /// * `id` - the ID of the user
    /// * `secret` - the TOTP secret
    /// * `recovery` - the TOTP recovery codes
    pub async fn update_user_totp(
        &self,
        id: usize,
        secret: &str,
        recovery: &Vec<String>,
    ) -> Result<()> {
        let user = self.get_user_by_id(id).await?;

        // update
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "UPDATE users SET totp = $1, recovery_codes = $2 WHERE id = $3",
            params![
                &secret,
                &serde_json::to_string(recovery).unwrap(),
                &(id as i64)
            ]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.cache_clear_user(&user).await;
        Ok(())
    }

    /// Enable TOTP for a profile.
    ///
    /// # Arguments
    /// * `id` - the ID of the user to enable TOTP for
    /// * `user` - the user doing this
    ///
    /// # Returns
    /// `Result<(secret, qr base64)>`
    pub async fn enable_totp(
        &self,
        id: usize,
        user: User,
    ) -> Result<(String, String, Vec<String>)> {
        let other_user = self.get_user_by_id(id).await?;

        if other_user.id != user.id {
            if other_user.permissions.check(FinePermission::MANAGE_USERS) {
                // create audit log entry
                self.create_audit_log_entry(AuditLogEntry::new(
                    user.id,
                    format!("invoked `enable_totp` with x value `{}`", other_user.id,),
                ))
                .await?;
            } else {
                return Err(Error::NotAllowed);
            }
        }

        let secret = totp_rs::Secret::default().to_string();
        let recovery = Self::generate_totp_recovery_codes();
        self.update_user_totp(id, &secret, &recovery).await?;

        // fetch profile again (with totp information)
        let other_user = self.get_user_by_id(id).await?;

        // get totp
        let totp = other_user.totp(Some(
            self.0
                .host
                .replace("http://", "")
                .replace("https://", "")
                .replace(":", "_"),
        ));

        if totp.is_none() {
            return Err(Error::MiscError("Failed to get TOTP code".to_string()));
        }

        let totp = totp.unwrap();

        // generate qr
        let qr = match totp.get_qr_base64() {
            Ok(q) => q,
            Err(e) => return Err(Error::MiscError(e.to_string())),
        };

        // return
        Ok((totp.get_secret_base32(), qr, recovery))
    }

    pub async fn cache_clear_user(&self, user: &User) {
        self.2.remove(format!("atto.user:{}", user.id)).await;
        self.2.remove(format!("atto.user:{}", user.username)).await;
    }

    auto_method!(update_user_tokens(Vec<Token>)@get_user_by_id -> "UPDATE users SET tokens = $1 WHERE id = $2" --serde --cache-key-tmpl=cache_clear_user);
    auto_method!(update_user_settings(UserSettings)@get_user_by_id -> "UPDATE users SET settings = $1 WHERE id = $2" --serde --cache-key-tmpl=cache_clear_user);

    auto_method!(incr_user_notifications()@get_user_by_id -> "UPDATE users SET notification_count = notification_count + 1 WHERE id = $1" --cache-key-tmpl=cache_clear_user --incr);
    auto_method!(decr_user_notifications()@get_user_by_id -> "UPDATE users SET notification_count = notification_count - 1 WHERE id = $1" --cache-key-tmpl=cache_clear_user --decr);

    auto_method!(incr_user_follower_count()@get_user_by_id -> "UPDATE users SET follower_count = follower_count + 1 WHERE id = $1" --cache-key-tmpl=cache_clear_user --incr);
    auto_method!(decr_user_follower_count()@get_user_by_id -> "UPDATE users SET follower_count = follower_count - 1 WHERE id = $1" --cache-key-tmpl=cache_clear_user --decr);

    auto_method!(incr_user_following_count()@get_user_by_id -> "UPDATE users SET following_count = following_count + 1 WHERE id = $1" --cache-key-tmpl=cache_clear_user --incr);
    auto_method!(decr_user_following_count()@get_user_by_id -> "UPDATE users SET following_count = following_count - 1 WHERE id = $1" --cache-key-tmpl=cache_clear_user --decr);

    auto_method!(incr_user_post_count()@get_user_by_id -> "UPDATE users SET post_count = post_count + 1 WHERE id = $1" --cache-key-tmpl=cache_clear_user --incr);
    auto_method!(decr_user_post_count()@get_user_by_id -> "UPDATE users SET post_count = post_count - 1 WHERE id = $1" --cache-key-tmpl=cache_clear_user --decr);

    auto_method!(update_user_request_count(i32)@get_user_by_id -> "UPDATE users SET request_count = $1 WHERE id = $2" --cache-key-tmpl=cache_clear_user);
    auto_method!(incr_user_request_count()@get_user_by_id -> "UPDATE users SET request_count = request_count + 1 WHERE id = $1" --cache-key-tmpl=cache_clear_user --incr);
    auto_method!(decr_user_request_count()@get_user_by_id -> "UPDATE users SET request_count = request_count - 1 WHERE id = $1" --cache-key-tmpl=cache_clear_user --decr);
}
