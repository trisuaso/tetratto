use super::*;
use crate::model::{
    Error, Result,
    auth::{Token, User},
    permissions::FinePermission,
};
use crate::{auto_method, execute, get, query_row};
use tetratto_shared::hash::hash_salted;

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
            id: get!(x->0(u64)) as usize,
            created: get!(x->1(u64)) as usize,
            username: get!(x->2(String)),
            password: get!(x->3(String)),
            salt: get!(x->4(String)),
            settings: serde_json::from_str(&get!(x->5(String)).to_string()).unwrap(),
            tokens: serde_json::from_str(&get!(x->6(String)).to_string()).unwrap(),
            permissions: FinePermission::from_bits(get!(x->7(u32))).unwrap(),
        }
    }

    auto_method!(get_user_by_id(&str)@get_user_from_row -> "SELECT * FROM users WHERE id = $1" --name="user" --returns=User);
    auto_method!(get_user_by_username(&str)@get_user_from_row -> "SELECT * FROM users WHERE username = $1" --name="user" --returns=User);

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
    pub async fn create_user(&self, data: User) -> Result<()> {
        if self.0.security.registration_enabled == false {
            return Err(Error::RegistrationDisabled);
        }

        // check values
        if data.username.len() < 2 {
            return Err(Error::DataTooShort("username".to_string()));
        } else if data.username.len() > 32 {
            return Err(Error::DataTooLong("username".to_string()));
        }

        if data.password.len() < 6 {
            return Err(Error::DataTooShort("password".to_string()));
        }

        // ...
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "INSERT INTO users VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            &[
                &data.id.to_string().as_str(),
                &data.created.to_string().as_str(),
                &data.username.as_str(),
                &data.password.as_str(),
                &data.salt.as_str(),
                &serde_json::to_string(&data.settings).unwrap().as_str(),
                &serde_json::to_string(&data.tokens).unwrap().as_str(),
                &(FinePermission::DEFAULT.bits()).to_string().as_str()
            ]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
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

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(&conn, "DELETE FROM users WHERE id = $1", &[&id]);

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        Ok(())
    }

    auto_method!(update_user_tokens(Vec<Token>) -> "UPDATE users SET tokens = $1 WHERE id = $2" --serde);
}
