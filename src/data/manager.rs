use super::model::{Error, Result, User};
use crate::config::Config;
use crate::write_template;

use pathbufd::PathBufD as PathBuf;
use rainbeam_shared::hash::hash_salted;
use rusqlite::{Connection, Result as SqlResult, Row};
use std::fs::{create_dir, exists};
use tera::{Context, Tera};

pub struct DataManager(pub(crate) Config, pub Tera);

impl DataManager {
    /// Obtain a connection to the staging database.
    pub(crate) fn connect(name: &str) -> SqlResult<Connection> {
        Ok(Connection::open(name)?)
    }

    /// Create a new [`DataManager`] (and init database).
    pub async fn new(config: Config) -> SqlResult<Self> {
        let conn = Self::connect(&config.database)?;

        conn.pragma_update(None, "journal_mode", "WAL").unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id       INTEGER NOT NULL PRIMARY KEY,
                created  INTEGER NOT NULL,
                username TEXT NOT NULL,
                password TEXT NOT NULL,
                salt     TEXT NOT NULL,
                settings TEXT NOT NULL,
                tokens   TEXT NOT NULL
            )",
            (),
        )
        .unwrap();

        // create system templates
        let html_path = PathBuf::current().join(&config.dirs.templates);
        let atto_dir = html_path.join("_atto");

        if !exists(&atto_dir).unwrap() {
            create_dir(&atto_dir).unwrap();
        }

        write_template!(atto_dir->"root.html"(super::assets::ROOT));

        write_template!(atto_dir->"auth/base.html"(super::assets::AUTH_BASE) -d "auth");
        write_template!(atto_dir->"auth/login.html"(super::assets::LOGIN));
        write_template!(atto_dir->"auth/register.html"(super::assets::REGISTER));

        // return
        Ok(Self(
            config.clone(),
            Tera::new(&format!("{html_path}/**/*")).unwrap(),
        ))
    }

    /// Create the initial template context.
    pub(crate) fn initial_context(&self) -> Context {
        let mut ctx = Context::new();
        ctx.insert("name", &self.0.name);
        ctx
    }

    // users

    /// Get a [`User`] from an SQL row.
    pub(crate) fn get_user_from_row(x: &Row<'_>) -> User {
        User {
            id: x.get(0).unwrap(),
            created: x.get(1).unwrap(),
            username: x.get(2).unwrap(),
            password: x.get(3).unwrap(),
            salt: x.get(4).unwrap(),
            settings: serde_json::from_str(&x.get::<usize, String>(5).unwrap().to_string())
                .unwrap(),
            tokens: serde_json::from_str(&x.get::<usize, String>(6).unwrap().to_string()).unwrap(),
        }
    }

    /// Get a user given just their `id`.
    ///
    /// # Arguments
    /// * `id` - the ID of the user
    pub async fn get_user_by_id(&self, id: &str) -> Result<User> {
        let conn = match Self::connect(&self.0.name) {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let mut query = conn.prepare("SELECT * FROM users WHERE id = ?").unwrap();
        let res = query.query_row([id], |x| Ok(Self::get_user_from_row(x)));

        if res.is_err() {
            return Err(Error::UserNotFound);
        }

        Ok(res.unwrap())
    }

    /// Get a user given just their auth token.
    ///
    /// # Arguments
    /// * `token` - the token of the user
    pub async fn get_user_by_token(&self, token: &str) -> Result<User> {
        let conn = match Self::connect(&self.0.name) {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let mut query = conn
            .prepare("SELECT * FROM users WHERE tokens LIKE ?")
            .unwrap();
        let res = query.query_row([format!("%,\"{token}\"%")], |x| {
            Ok(Self::get_user_from_row(x))
        });

        if res.is_err() {
            return Err(Error::UserNotFound);
        }

        Ok(res.unwrap())
    }

    /// Create a new user in the database.
    ///
    /// # Arguments
    /// * `data` - a mock [`User`] object to insert
    pub async fn create_user(&self, data: User) -> Result<()> {
        if self.0.security.registration_enabled == false {
            return Err(Error::RegistrationDisabled);
        }

        let conn = match Self::connect(&self.0.name) {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = conn.execute(
            "INSERT INTO users VALUES (?, ?, ?, ?, ?, ?, ?)",
            (
                data.id,
                data.created,
                data.username,
                data.password,
                data.salt,
                serde_json::to_string(&data.settings).unwrap(),
                serde_json::to_string(&data.tokens).unwrap(),
            ),
        );

        if res.is_err() {
            return Err(Error::DatabaseError);
        }

        Ok(())
    }

    /// Create a new user in the database.
    ///
    /// # Arguments
    /// * `id` - the ID of the user
    /// * `password` - the current password of the user
    /// * `force` - if we should delete even if the given password is incorrect
    pub async fn delete_user(&self, id: &str, password: &str, force: bool) -> Result<()> {
        let user = self.get_user_by_id(id).await?;

        if (hash_salted(password.to_string(), user.salt) != user.password) && !force {
            return Err(Error::IncorrectPassword);
        }

        let conn = match Self::connect(&self.0.name) {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = conn.execute("DELETE FROM users WHERE id = ?", [id]);

        if res.is_err() {
            return Err(Error::DatabaseError);
        }

        Ok(())
    }
}
