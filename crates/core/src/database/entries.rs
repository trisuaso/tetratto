use super::*;
use crate::cache::Cache;
use crate::model::journal::JournalEntryContext;
use crate::model::{
    Error, Result, auth::User, journal::JournalEntry, journal::JournalPageWriteAccess,
    permissions::FinePermission,
};
use crate::{auto_method, execute, get, query_row};

#[cfg(feature = "sqlite")]
use rusqlite::Row;

#[cfg(feature = "postgres")]
use tokio_postgres::Row;

impl DataManager {
    /// Get a [`JournalEntry`] from an SQL row.
    pub(crate) fn get_entry_from_row(
        #[cfg(feature = "sqlite")] x: &Row<'_>,
        #[cfg(feature = "postgres")] x: &Row,
    ) -> JournalEntry {
        JournalEntry {
            id: get!(x->0(u64)) as usize,
            created: get!(x->1(u64)) as usize,
            content: get!(x->2(String)),
            owner: get!(x->3(u64)) as usize,
            journal: get!(x->4(u64)) as usize,
            context: serde_json::from_str(&get!(x->5(String))).unwrap(),
            // likes
            likes: get!(x->6(i64)) as isize,
            dislikes: get!(x->7(i64)) as isize,
        }
    }

    auto_method!(get_entry_by_id()@get_entry_from_row -> "SELECT * FROM entries WHERE id = $1" --name="journal entry" --returns=JournalEntry --cache-key-tmpl="atto.entry:{}");

    /// Create a new journal entry in the database.
    ///
    /// # Arguments
    /// * `data` - a mock [`JournalEntry`] object to insert
    pub async fn create_entry(&self, data: JournalEntry) -> Result<()> {
        // check values
        if data.content.len() < 2 {
            return Err(Error::DataTooShort("content".to_string()));
        } else if data.content.len() > 4096 {
            return Err(Error::DataTooLong("username".to_string()));
        }

        // check permission in page
        let page = match self.get_page_by_id(data.journal).await {
            Ok(p) => p,
            Err(e) => return Err(e),
        };

        match page.write_access {
            JournalPageWriteAccess::Owner => {
                if data.owner != page.owner {
                    return Err(Error::NotAllowed);
                }
            }
            JournalPageWriteAccess::Joined => {
                if let Err(_) = self
                    .get_membership_by_owner_journal(data.owner, page.id)
                    .await
                {
                    return Err(Error::NotAllowed);
                }
            }
            _ => (),
        };

        // ...
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "INSERT INTO entries VALUES ($1, $2, $3, $4, $5",
            &[
                &data.id.to_string().as_str(),
                &data.created.to_string().as_str(),
                &data.content.as_str(),
                &data.owner.to_string().as_str(),
                &data.journal.to_string().as_str(),
                &serde_json::to_string(&data.context).unwrap().as_str(),
            ]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        Ok(())
    }

    auto_method!(delete_entry()@get_entry_by_id:MANAGE_JOURNAL_ENTRIES -> "DELETE FROM entries WHERE id = $1" --cache-key-tmpl="atto.entry:{}");
    auto_method!(update_entry_content(String)@get_entry_by_id:MANAGE_JOURNAL_ENTRIES -> "UPDATE entries SET content = $1 WHERE id = $2" --cache-key-tmpl="atto.entry:{}");
    auto_method!(update_entry_context(JournalEntryContext)@get_entry_by_id:MANAGE_JOURNAL_ENTRIES -> "UPDATE entries SET context = $1 WHERE id = $2" --serde --cache-key-tmpl="atto.entry:{}");

    auto_method!(incr_entry_likes() -> "UPDATE entries SET likes = likes + 1 WHERE id = $1" --cache-key-tmpl="atto.entry:{}" --reactions-key-tmpl="atto.entry.likes:{}" --incr);
    auto_method!(incr_entry_dislikes() -> "UPDATE entries SET likes = dislikes + 1 WHERE id = $1" --cache-key-tmpl="atto.entry:{}" --reactions-key-tmpl="atto.entry.dislikes:{}" --incr);
    auto_method!(decr_entry_likes() -> "UPDATE entries SET likes = likes - 1 WHERE id = $1" --cache-key-tmpl="atto.entry:{}" --reactions-key-tmpl="atto.entry.likes:{}" --decr);
    auto_method!(decr_entry_dislikes() -> "UPDATE entries SET likes = dislikes - 1 WHERE id = $1" --cache-key-tmpl="atto.entry:{}" --reactions-key-tmpl="atto.entry.dislikes:{}" --decr);
}
