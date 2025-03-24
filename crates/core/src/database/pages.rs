use super::*;
use crate::cache::Cache;
use crate::model::auth::User;
use crate::model::{Error, Result, journal::JournalPage, permissions::FinePermission};
use crate::{auto_method, execute, get, query_row};

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

    auto_method!(get_page_by_id()@get_page_from_row -> "SELECT * FROM pages WHERE id = $1" --name="journal page" --returns=JournalPage --cache-key-tmpl="atto.page:{}");

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

    auto_method!(delete_page()@get_page_by_id:MANAGE_JOURNAL_PAGES -> "DELETE FROM pages WHERE id = $1" --cache-key-tmpl="atto.page:{}");
    auto_method!(update_page_title(String)@get_page_by_id:MANAGE_JOURNAL_PAGES -> "UPDATE pages SET title = $1 WHERE id = $2" --cache-key-tmpl="atto.page:{}");
    auto_method!(update_page_prompt(String)@get_page_by_id:MANAGE_JOURNAL_PAGES -> "UPDATE pages SET prompt = $1 WHERE id = $2" --cache-key-tmpl="atto.page:{}");
    auto_method!(update_page_read_access(String)@get_page_by_id:MANAGE_JOURNAL_PAGES -> "UPDATE pages SET read_access = $1 WHERE id = $2" --serde --cache-key-tmpl="atto.page:{}");
    auto_method!(update_page_write_access(String)@get_page_by_id:MANAGE_JOURNAL_PAGES -> "UPDATE pages SET write_access = $1 WHERE id = $2" --serde --cache-key-tmpl="atto.page:{}");
}
