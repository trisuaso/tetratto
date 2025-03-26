use super::*;
use crate::cache::Cache;
use crate::model::journal::JournalMembership;
use crate::model::journal_permissions::JournalPermission;
use crate::model::{
    Error, Result,
    auth::User,
    journal::Journal,
    journal::{JournalReadAccess, JournalWriteAccess},
    permissions::FinePermission,
};
use crate::{auto_method, execute, get, query_row};

#[cfg(feature = "sqlite")]
use rusqlite::Row;

#[cfg(feature = "postgres")]
use tokio_postgres::Row;

impl DataManager {
    /// Get a [`Journal`] from an SQL row.
    pub(crate) fn get_page_from_row(
        #[cfg(feature = "sqlite")] x: &Row<'_>,
        #[cfg(feature = "postgres")] x: &Row,
    ) -> Journal {
        Journal {
            id: get!(x->0(i64)) as usize,
            created: get!(x->1(i64)) as usize,
            title: get!(x->2(String)),
            prompt: get!(x->3(String)),
            owner: get!(x->4(i64)) as usize,
            read_access: serde_json::from_str(&get!(x->5(String)).to_string()).unwrap(),
            write_access: serde_json::from_str(&get!(x->6(String)).to_string()).unwrap(),
            // likes
            likes: get!(x->6(i64)) as isize,
            dislikes: get!(x->7(i64)) as isize,
        }
    }

    auto_method!(get_page_by_id()@get_page_from_row -> "SELECT * FROM pages WHERE id = $1" --name="journal page" --returns=Journal --cache-key-tmpl="atto.page:{}");

    /// Create a new journal page in the database.
    ///
    /// # Arguments
    /// * `data` - a mock [`Journal`] object to insert
    pub async fn create_page(&self, data: Journal) -> Result<()> {
        // check values
        if data.title.len() < 2 {
            return Err(Error::DataTooShort("title".to_string()));
        } else if data.title.len() > 32 {
            return Err(Error::DataTooLong("title".to_string()));
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

        // add journal page owner as admin
        self.create_membership(JournalMembership::new(
            data.owner,
            data.id,
            JournalPermission::ADMINISTRATOR,
        ))
        .await
        .unwrap();

        // return
        Ok(())
    }

    auto_method!(delete_page()@get_page_by_id:MANAGE_JOURNAL_PAGES -> "DELETE FROM pages WHERE id = $1" --cache-key-tmpl="atto.page:{}");
    auto_method!(update_page_title(String)@get_page_by_id:MANAGE_JOURNAL_PAGES -> "UPDATE pages SET title = $1 WHERE id = $2" --cache-key-tmpl="atto.page:{}");
    auto_method!(update_page_prompt(String)@get_page_by_id:MANAGE_JOURNAL_PAGES -> "UPDATE pages SET prompt = $1 WHERE id = $2" --cache-key-tmpl="atto.page:{}");
    auto_method!(update_page_read_access(JournalReadAccess)@get_page_by_id:MANAGE_JOURNAL_PAGES -> "UPDATE pages SET read_access = $1 WHERE id = $2" --serde --cache-key-tmpl="atto.page:{}");
    auto_method!(update_page_write_access(JournalWriteAccess)@get_page_by_id:MANAGE_JOURNAL_PAGES -> "UPDATE pages SET write_access = $1 WHERE id = $2" --serde --cache-key-tmpl="atto.page:{}");

    auto_method!(incr_page_likes() -> "UPDATE pages SET likes = likes + 1 WHERE id = $1" --cache-key-tmpl="atto.pages:{}"  --incr);
    auto_method!(incr_page_dislikes() -> "UPDATE pages SET likes = dislikes + 1 WHERE id = $1" --cache-key-tmpl="atto.pages:{}" --incr);
    auto_method!(decr_page_likes() -> "UPDATE pages SET likes = likes - 1 WHERE id = $1" --cache-key-tmpl="atto.pages:{}" --decr);
    auto_method!(decr_page_dislikes() -> "UPDATE pages SET likes = dislikes - 1 WHERE id = $1" --cache-key-tmpl="atto.pages:{}" --decr);
}
