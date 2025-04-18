use std::collections::HashMap;

use super::*;
use crate::cache::Cache;
use crate::model::communities_permissions::CommunityPermission;
use crate::model::{
    Error, Result,
    communities::Question,
    requests::{ActionRequest, ActionType},
    auth::User,
    permissions::FinePermission,
};
use crate::{auto_method, execute, get, query_row, query_rows, params};

#[cfg(feature = "sqlite")]
use rusqlite::Row;

use tetratto_shared::unix_epoch_timestamp;
#[cfg(feature = "postgres")]
use tokio_postgres::Row;

impl DataManager {
    /// Get a [`Question`] from an SQL row.
    pub(crate) fn get_question_from_row(
        #[cfg(feature = "sqlite")] x: &Row<'_>,
        #[cfg(feature = "postgres")] x: &Row,
    ) -> Question {
        Question {
            id: get!(x->0(i64)) as usize,
            created: get!(x->1(i64)) as usize,
            owner: get!(x->2(i64)) as usize,
            receiver: get!(x->3(i64)) as usize,
            content: get!(x->4(String)),
            is_global: get!(x->5(i32)) as i8 == 1,
            answer_count: get!(x->6(i32)) as usize,
            community: get!(x->7(i64)) as usize,
            // likes
            likes: get!(x->8(i32)) as isize,
            dislikes: get!(x->9(i32)) as isize,
            // ...
            context: serde_json::from_str(&get!(x->10(String))).unwrap(),
            ip: get!(x->11(String)),
        }
    }

    auto_method!(get_question_by_id()@get_question_from_row -> "SELECT * FROM questions WHERE id = $1" --name="question" --returns=Question --cache-key-tmpl="atto.question:{}");

    /// Fill the given vector of questions with their owner as well.
    pub async fn fill_questions(&self, questions: Vec<Question>) -> Result<Vec<(Question, User)>> {
        let mut out: Vec<(Question, User)> = Vec::new();

        let mut seen_users: HashMap<usize, User> = HashMap::new();
        for question in questions {
            if let Some(ua) = seen_users.get(&question.owner) {
                out.push((question, ua.to_owned()));
            } else {
                let user = if question.owner == 0 {
                    User::anonymous()
                } else {
                    self.get_user_by_id_with_void(question.owner).await?
                };

                seen_users.insert(question.owner, user.clone());
                out.push((question, user));
            }
        }

        Ok(out)
    }

    /// Get all questions by `owner`.
    pub async fn get_questions_by_owner(&self, owner: usize) -> Result<Vec<Question>> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_rows!(
            &conn,
            "SELECT * FROM questions WHERE owner = $1 AND NOT context LIKE '%\"is_nsfw\":true%' ORDER BY created DESC",
            &[&(owner as i64)],
            |x| { Self::get_question_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("question".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Get all questions by `receiver`.
    pub async fn get_questions_by_receiver(&self, receiver: usize) -> Result<Vec<Question>> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_rows!(
            &conn,
            "SELECT * FROM questions WHERE receiver = $1 ORDER BY created DESC",
            &[&(receiver as i64)],
            |x| { Self::get_question_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("question".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Get all global questions by `community`.
    pub async fn get_questions_by_community(
        &self,
        community: usize,
        batch: usize,
        page: usize,
    ) -> Result<Vec<Question>> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_rows!(
            &conn,
            "SELECT * FROM questions WHERE community = $1 AND is_global = 1 ORDER BY created DESC LIMIT $2 OFFSET $3",
            &[
                &(community as i64),
                &(batch as i64),
                &((page * batch) as i64)
            ],
            |x| { Self::get_question_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("question".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Get all global questions by the given user's following.
    pub async fn get_questions_from_user_following(
        &self,
        id: usize,
        batch: usize,
        page: usize,
    ) -> Result<Vec<Question>> {
        let following = self.get_userfollows_by_initiator_all(id).await?;
        let mut following = following.iter();
        let first = match following.next() {
            Some(f) => f,
            None => return Ok(Vec::new()),
        };

        let mut query_string: String = String::new();

        for user in following {
            query_string.push_str(&format!(" OR owner = {}", user.receiver));
        }

        // ...
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_rows!(
            &conn,
            &format!(
                "SELECT * FROM questions WHERE (owner = {} {query_string}) AND is_global = 1 ORDER BY created DESC LIMIT $1 OFFSET $2",
                first.receiver
            ),
            &[&(batch as i64), &((page * batch) as i64)],
            |x| { Self::get_question_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("question".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Get all global questions posted in the given user's communities.
    pub async fn get_questions_from_user_communities(
        &self,
        id: usize,
        batch: usize,
        page: usize,
    ) -> Result<Vec<Question>> {
        let memberships = self.get_memberships_by_owner(id).await?;
        let mut memberships = memberships.iter();
        let first = match memberships.next() {
            Some(f) => f,
            None => return Ok(Vec::new()),
        };

        let mut query_string: String = String::new();

        for membership in memberships {
            query_string.push_str(&format!(" OR community = {}", membership.community));
        }

        // ...
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_rows!(
            &conn,
            &format!(
                "SELECT * FROM questions WHERE (community = {} {query_string}) AND is_global = 1 AND NOT context LIKE '%\"is_nsfw\":true%' ORDER BY created DESC LIMIT $1 OFFSET $2",
                first.community
            ),
            &[&(batch as i64), &((page * batch) as i64)],
            |x| { Self::get_question_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("question".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Get global questions from all communities, sorted by creation.
    ///
    /// # Arguments
    /// * `batch` - the limit of questions in each page
    /// * `page` - the page number
    pub async fn get_latest_global_questions(
        &self,
        batch: usize,
        page: usize,
    ) -> Result<Vec<Question>> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_rows!(
            &conn,
            "SELECT * FROM questions WHERE is_global = 1 ORDER BY created DESC LIMIT $1 OFFSET $2",
            &[&(batch as i64), &((page * batch) as i64)],
            |x| { Self::get_question_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("question".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Get global questions from all communities, sorted by likes.
    ///
    /// # Arguments
    /// * `batch` - the limit of questions in each page
    /// * `page` - the page number
    /// * `cutoff` - the maximum number of milliseconds ago the question could have been created
    pub async fn get_popular_global_questions(
        &self,
        batch: usize,
        page: usize,
        cutoff: usize,
    ) -> Result<Vec<Question>> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = query_rows!(
            &conn,
            "SELECT * FROM questions WHERE is_global = 1 AND NOT context LIKE '%\"is_nsfw\":true%' AND ($1 - created) < $2 ORDER BY likes DESC, created ASC LIMIT $3 OFFSET $4",
            &[
                &(unix_epoch_timestamp() as i64),
                &(cutoff as i64),
                &(batch as i64),
                &((page * batch) as i64)
            ],
            |x| { Self::get_question_from_row(x) }
        );

        if res.is_err() {
            return Err(Error::GeneralNotFound("question".to_string()));
        }

        Ok(res.unwrap())
    }

    /// Create a new question in the database.
    ///
    /// # Arguments
    /// * `data` - a mock [`Question`] object to insert
    pub async fn create_question(&self, mut data: Question) -> Result<usize> {
        // check if we can post this
        if data.is_global {
            if data.community > 0 {
                // posting to community
                data.receiver = 0;
                let community = self.get_community_by_id(data.community).await?;

                if !community.context.enable_questions
                    | !self.check_can_post(&community, data.owner).await
                {
                    return Err(Error::QuestionsDisabled);
                }

                // inherit nsfw status
                data.context.is_nsfw = community.context.is_nsfw;
            } else {
                let receiver = self.get_user_by_id(data.receiver).await?;

                if !receiver.settings.enable_questions {
                    return Err(Error::QuestionsDisabled);
                }

                // check for ip block
                if self
                    .get_ipblock_by_initiator_receiver(receiver.id, &data.ip)
                    .await
                    .is_ok()
                {
                    return Err(Error::NotAllowed);
                }
            }
        } else {
            let receiver = self.get_user_by_id(data.receiver).await?;

            if !receiver.settings.enable_questions {
                return Err(Error::QuestionsDisabled);
            }

            if !receiver.settings.allow_anonymous_questions && data.owner == 0 {
                return Err(Error::NotAllowed);
            }

            // check for ip block
            if self
                .get_ipblock_by_initiator_receiver(receiver.id, &data.ip)
                .await
                .is_ok()
            {
                return Err(Error::NotAllowed);
            }
        }

        // ...
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "INSERT INTO questions VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
            params![
                &(data.id as i64),
                &(data.created as i64),
                &(data.owner as i64),
                &(data.receiver as i64),
                &data.content,
                &{ if data.is_global { 1 } else { 0 } },
                &0_i32,
                &(data.community as i64),
                &0_i32,
                &0_i32,
                &serde_json::to_string(&data.context).unwrap(),
                &data.ip
            ]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // create request
        if !data.is_global {
            self.create_request(ActionRequest::with_id(
                data.owner,
                data.receiver,
                ActionType::Answer,
                data.id,
            ))
            .await?;
        }

        // return
        Ok(data.id)
    }

    pub async fn delete_question(&self, id: usize, user: &User) -> Result<()> {
        let y = self.get_question_by_id(id).await?;

        if user.id != y.owner
            && user.id != y.receiver
            && !user.permissions.check(FinePermission::MANAGE_QUESTIONS)
        {
            if y.community != 0 {
                // check for MANAGE_QUESTIONS permission
                let membership = self
                    .get_membership_by_owner_community_no_void(user.id, y.community)
                    .await?;

                if !membership.role.check(CommunityPermission::MANAGE_QUESTIONS) {
                    return Err(Error::NotAllowed);
                }
            } else {
                return Err(Error::NotAllowed);
            }
        }

        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        let res = execute!(
            &conn,
            "DELETE FROM questions WHERE id = $1",
            &[&(id as i64)]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        self.2.remove(format!("atto.question:{}", id)).await;

        // delete request (if it exists and question isn't global)
        if !y.is_global
            && self
                .get_request_by_id_linked_asset(y.owner, y.id)
                .await
                .is_ok()
        {
            // requests are also deleted when a post is created answering the given question
            // (unless the question is global)
            self.delete_request(y.owner, y.id, user, false).await?;
        }

        // delete all posts answering question
        let res = execute!(
            &conn,
            "DELETE FROM posts WHERE context LIKE $1",
            &[&format!("%\"answering\":{id}%")]
        );

        if let Err(e) = res {
            return Err(Error::DatabaseError(e.to_string()));
        }

        // return
        Ok(())
    }

    pub async fn delete_all_questions(&self, user: &User) -> Result<()> {
        let y = self.get_questions_by_receiver(user.id).await?;

        for x in y {
            if user.id != x.receiver && !user.permissions.check(FinePermission::MANAGE_QUESTIONS) {
                return Err(Error::NotAllowed);
            }

            self.delete_question(x.id, user).await?
        }

        Ok(())
    }

    auto_method!(incr_question_answer_count() -> "UPDATE questions SET answer_count = answer_count + 1 WHERE id = $1" --cache-key-tmpl="atto.question:{}" --incr);
    auto_method!(decr_question_answer_count() -> "UPDATE questions SET answer_count = answer_count - 1 WHERE id = $1" --cache-key-tmpl="atto.question:{}" --decr);

    auto_method!(incr_question_likes() -> "UPDATE questions SET likes = likes + 1 WHERE id = $1" --cache-key-tmpl="atto.question:{}" --incr);
    auto_method!(incr_question_dislikes() -> "UPDATE questions SET dislikes = dislikes + 1 WHERE id = $1" --cache-key-tmpl="atto.question:{}" --incr);
    auto_method!(decr_question_likes() -> "UPDATE questions SET likes = likes - 1 WHERE id = $1" --cache-key-tmpl="atto.question:{}" --decr);
    auto_method!(decr_question_dislikes() -> "UPDATE questions SET dislikes = dislikes - 1 WHERE id = $1" --cache-key-tmpl="atto.question:{}" --decr);
}
