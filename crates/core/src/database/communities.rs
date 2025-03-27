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

    auto_method!(get_community_by_id()@get_community_from_row -> "SELECT * FROM communities WHERE id = $1" --name="community" --returns=Community --cache-key-tmpl="atto.community:{}");

    /// Create a new community in the database.
    ///
    /// # Arguments
    /// * `data` - a mock [`Community`] to insert
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
            "INSERT INTO communities VALUES ($1, $2, $3, $4, $5, $6, $7)",
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

    auto_method!(delete_community()@get_community_by_id:MANAGE_COMMUNITY_PAGES -> "DELETE communities pages WHERE id = $1" --cache-key-tmpl="atto.community:{}");
    auto_method!(update_community_title(String)@get_community_by_id:MANAGE_COMMUNITY_PAGES -> "UPDATE communities SET title = $1 WHERE id = $2" --cache-key-tmpl="atto.community:{}");
    auto_method!(update_community_context(CommunityContext)@get_community_by_id:MANAGE_COMMUNITY_PAGES -> "UPDATE communities SET prompt = $1 WHERE id = $2" --serde --cache-key-tmpl="atto.community:{}");
    auto_method!(update_community_read_access(CommunityReadAccess)@get_community_by_id:MANAGE_COMMUNITY_PAGES -> "UPDATE communities SET read_access = $1 WHERE id = $2" --serde --cache-key-tmpl="atto.community:{}");
    auto_method!(update_community_write_access(CommunityWriteAccess)@get_community_by_id:MANAGE_COMMUNITY_PAGES -> "UPDATE communities SET write_access = $1 WHERE id = $2" --serde --cache-key-tmpl="atto.community:{}");

    auto_method!(incr_community_likes() -> "UPDATE communities SET likes = likes + 1 WHERE id = $1" --cache-key-tmpl="atto.community:{}"  --incr);
    auto_method!(incr_community_dislikes() -> "UPDATE communities SET likes = dislikes + 1 WHERE id = $1" --cache-key-tmpl="atto.community:{}" --incr);
    auto_method!(decr_community_likes() -> "UPDATE communities SET likes = likes - 1 WHERE id = $1" --cache-key-tmpl="atto.community:{}" --decr);
    auto_method!(decr_community_dislikes() -> "UPDATE communities SET likes = dislikes - 1 WHERE id = $1" --cache-key-tmpl="atto.community:{}" --decr);
}
