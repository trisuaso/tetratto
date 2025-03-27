use super::*;
use crate::cache::Cache;
use crate::model::communities::{CommunityContext, CommunityMembership};
use crate::model::communities_permissions::CommunityPermission;
use crate::model::{
    Error, Result,
    auth::User,
    communities::Community,
    communities::{CommunityReadAccess, CommunityWriteAccess},
    permissions::FinePermission,
};
use crate::{auto_method, execute, get, query_row};

#[cfg(feature = "sqlite")]
use rusqlite::Row;

#[cfg(feature = "postgres")]
use tokio_postgres::Row;

impl DataManager {
    /// Get a [`Community`] from an SQL row.
    pub(crate) fn get_community_from_row(
        #[cfg(feature = "sqlite")] x: &Row<'_>,
        #[cfg(feature = "postgres")] x: &Row,
    ) -> Community {
        Community {
            id: get!(x->0(i64)) as usize,
            created: get!(x->1(i64)) as usize,
            title: get!(x->2(String)),
            context: serde_json::from_str(&get!(x->3(String))).unwrap(),
            owner: get!(x->4(i64)) as usize,
            read_access: serde_json::from_str(&get!(x->5(String))).unwrap(),
            write_access: serde_json::from_str(&get!(x->6(String))).unwrap(),
            // likes
            likes: get!(x->6(i64)) as isize,
            dislikes: get!(x->7(i64)) as isize,
        }
    }

    auto_method!(get_page_by_id()@get_community_from_row -> "SELECT * FROM journals WHERE id = $1" --name="journal" --returns=Community --cache-key-tmpl="atto.journal:{}");

    /// Create a new journal page in the database.
    ///
    /// # Arguments
    /// * `data` - a mock [`Journal`] object to insert
    pub async fn create_community(&self, data: Community) -> Result<()> {
        // check values
        if data.title.len() < 2 {
            return Err(Error::DataTooShort("title".to_string()));
        } else if data.title.len() > 32 {
            return Err(Error::DataTooLong("title".to_string()));
        }

        // ...
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "INSERT INTO journals VALUES ($1, $2, $3, $4, $5, $6, $7)",
            &[
                &data.id.to_string().as_str(),
                &data.created.to_string().as_str(),
                &data.title.as_str(),
                &serde_json::to_string(&data.context).unwrap().as_str(),
                &data.owner.to_string().as_str(),
                &serde_json::to_string(&data.read_access).unwrap().as_str(),
                &serde_json::to_string(&data.write_access).unwrap().as_str(),
            ]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // add journal page owner as admin
        self.create_membership(CommunityMembership::new(
            data.owner,
            data.id,
            CommunityPermission::ADMINISTRATOR,
        ))
        .await
        .unwrap();

        // return
        Ok(())
    }

    auto_method!(delete_community()@get_page_by_id:MANAGE_JOURNAL_PAGES -> "DELETE journals pages WHERE id = $1" --cache-key-tmpl="atto.journal:{}");
    auto_method!(update_community_title(String)@get_page_by_id:MANAGE_JOURNAL_PAGES -> "UPDATE journals SET title = $1 WHERE id = $2" --cache-key-tmpl="atto.journal:{}");
    auto_method!(update_community_context(CommunityContext)@get_page_by_id:MANAGE_JOURNAL_PAGES -> "UPDATE journals SET prompt = $1 WHERE id = $2" --serde --cache-key-tmpl="atto.journal:{}");
    auto_method!(update_community_read_access(CommunityReadAccess)@get_page_by_id:MANAGE_JOURNAL_PAGES -> "UPDATE journals SET read_access = $1 WHERE id = $2" --serde --cache-key-tmpl="atto.journal:{}");
    auto_method!(update_community_write_access(CommunityWriteAccess)@get_page_by_id:MANAGE_JOURNAL_PAGES -> "UPDATE journals SET write_access = $1 WHERE id = $2" --serde --cache-key-tmpl="atto.journal:{}");

    auto_method!(incr_page_likes() -> "UPDATE journals SET likes = likes + 1 WHERE id = $1" --cache-key-tmpl="atto.journal:{}"  --incr);
    auto_method!(incr_page_dislikes() -> "UPDATE journals SET likes = dislikes + 1 WHERE id = $1" --cache-key-tmpl="atto.journal:{}" --incr);
    auto_method!(decr_page_likes() -> "UPDATE journals SET likes = likes - 1 WHERE id = $1" --cache-key-tmpl="atto.journal:{}" --decr);
    auto_method!(decr_page_dislikes() -> "UPDATE journals SET likes = dislikes - 1 WHERE id = $1" --cache-key-tmpl="atto.journal:{}" --decr);
}
