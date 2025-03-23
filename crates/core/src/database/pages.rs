use super::*;
use crate::model::auth::User;
use crate::model::{Error, Result, journal::JournalPage, permissions::FinePermission};
use crate::{execute, get, query_row};

#[cfg(feature = "sqlite")]
use rusqlite::Row;

#[cfg(feature = "postgres")]
use tokio_postgres::Row;

impl DataManager {
    /// Get a [`JournalPage`] from an SQL row.
    pub(crate) fn get_page_from_row(
        #[cfg(feature = "sqlite")] x: &Row<'_>,
        #[cfg(feature = "postgres")] x: &Row,
    ) -> JournalPage {
        JournalPage {
            id: get!(x->0(u64)) as usize,
            created: get!(x->1(u64)) as usize,
            title: get!(x->2(String)),
            prompt: get!(x->3(String)),
            owner: get!(x->4(u64)) as usize,
            read_access: serde_json::from_str(&get!(x->5(String)).to_string()).unwrap(),
            write_access: serde_json::from_str(&get!(x->6(String)).to_string()).unwrap(),
        }
    }

    /// Get a journal page given just its `id`.
    ///
    /// # Arguments
    /// * `id` - the ID of the page
    pub async fn get_page_by_id(&self, id: &str) -> Result<JournalPage> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_row!(&conn, "SELECT * FROM pages WHERE id = $1", &[&id], |x| {
            Ok(Self::get_page_from_row(x))
        });

        if res.is_err() {
            return Err(Error::GeneralNotFound("journal page".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Create a new journal page in the database.
    ///
    /// # Arguments
    /// * `data` - a mock [`JournalPage`] object to insert
    pub async fn create_page(&self, data: JournalPage) -> Result<()> {
        if self.0.security.registration_enabled == false {
            return Err(Error::RegistrationDisabled);
        }

        // check values
        if data.title.len() < 2 {
            return Err(Error::DataTooShort("title".to_string()));
        } else if data.title.len() > 32 {
            return Err(Error::DataTooLong("username".to_string()));
        }

        if data.prompt.len() < 2 {
            return Err(Error::DataTooShort("prompt".to_string()));
        } else if data.prompt.len() > 2048 {
            return Err(Error::DataTooLong("prompt".to_string()));
        }

        // ...
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "INSERT INTO pages VALUES ($1, $2, $3, $4, $5, $6, $7)",
            &[
                &data.id.to_string().as_str(),
                &data.created.to_string().as_str(),
                &data.title.as_str(),
                &data.prompt.as_str(),
                &data.owner.to_string().as_str(),
                &serde_json::to_string(&data.read_access).unwrap().as_str(),
                &serde_json::to_string(&data.write_access).unwrap().as_str(),
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
    /// * `id` - the ID of the page
    /// * `user` - the user deleting the page
    pub async fn delete_page(&self, id: &str, user: User) -> Result<()> {
        let page = self.get_page_by_id(id).await?;

        if user.id != page.owner {
            if !user.permissions.check(FinePermission::MANAGE_JOURNAL_PAGES) {
                return Err(Error::NotAllowed);
            }
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(&conn, "DELETE FROM pages WHERE id = $1", &[&id]);

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        Ok(())
    }
}
