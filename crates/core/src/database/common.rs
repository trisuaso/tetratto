use crate::{
    database::drivers::common,
    execute,
    model::{Error, Result},
};

use super::DataManager;

impl DataManager {
    pub async fn init(&self) -> Result<()> {
        let conn = match self.connect().await {
            Ok(c) => c,
            Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
        };

        execute!(&conn, common::CREATE_TABLE_USERS).unwrap();
        execute!(&conn, common::CREATE_TABLE_COMMUNITIES).unwrap();
        execute!(&conn, common::CREATE_TABLE_POSTS).unwrap();
        execute!(&conn, common::CREATE_TABLE_MEMBERSHIPS).unwrap();
        execute!(&conn, common::CREATE_TABLE_REACTIONS).unwrap();
        execute!(&conn, common::CREATE_TABLE_NOTIFICATIONS).unwrap();
        execute!(&conn, common::CREATE_TABLE_USERFOLLOWS).unwrap();
        execute!(&conn, common::CREATE_TABLE_USERBLOCKS).unwrap();
        execute!(&conn, common::CREATE_TABLE_IPBANS).unwrap();
        execute!(&conn, common::CREATE_TABLE_AUDIT_LOG).unwrap();
        execute!(&conn, common::CREATE_TABLE_REPORTS).unwrap();
        execute!(&conn, common::CREATE_TABLE_USER_WARNINGS).unwrap();
        execute!(&conn, common::CREATE_TABLE_REQUESTS).unwrap();
        execute!(&conn, common::CREATE_TABLE_QUESTIONS).unwrap();
        execute!(&conn, common::CREATE_TABLE_IPBLOCKS).unwrap();

        Ok(())
    }
}

#[macro_export]
macro_rules! auto_method {
    ($name:ident()@$select_fn:ident -> $query:literal --name=$name_:literal --returns=$returns_:tt) => {
        pub async fn $name(&self, id: usize) -> Result<$returns_> {
            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = query_row!(&conn, $query, &[&(id as i64)], |x| {
                Ok(Self::$select_fn(x))
            });

            if res.is_err() {
                return Err(Error::GeneralNotFound($name_.to_string()));
            }

            Ok(res.unwrap())
        }
    };

    ($name:ident()@$select_fn:ident -> $query:literal --name=$name_:literal --returns=$returns_:tt --cache-key-tmpl=$cache_key_tmpl:literal) => {
        pub async fn $name(&self, id: usize) -> Result<$returns_> {
            if let Some(cached) = self.2.get(format!($cache_key_tmpl, id)).await {
                return Ok(serde_json::from_str(&cached).unwrap());
            }

            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = query_row!(&conn, $query, &[&(id as i64)], |x| {
                Ok(Self::$select_fn(x))
            });

            if res.is_err() {
                return Err(Error::GeneralNotFound($name_.to_string()));
            }

            let x = res.unwrap();
            self.2
                .set(
                    format!($cache_key_tmpl, id),
                    serde_json::to_string(&x).unwrap(),
                )
                .await;

            Ok(x)
        }
    };

    ($name:ident($selector_t:ty)@$select_fn:ident -> $query:literal --name=$name_:literal --returns=$returns_:tt) => {
        pub async fn $name(&self, selector: $selector_t) -> Result<$returns_> {
            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = query_row!(&conn, $query, &[&selector], |x| { Ok(Self::$select_fn(x)) });

            if res.is_err() {
                return Err(Error::GeneralNotFound($name_.to_string()));
            }

            Ok(res.unwrap())
        }
    };

    ($name:ident($selector_t:ty)@$select_fn:ident -> $query:literal --name=$name_:literal --returns=$returns_:tt --cache-key-tmpl=$cache_key_tmpl:literal) => {
        pub async fn $name(&self, selector: $selector_t) -> Result<$returns_> {
            let selector = selector.to_string().to_lowercase();

            if let Some(cached) = self.2.get(format!($cache_key_tmpl, selector)).await {
                return Ok(serde_json::from_str(&cached).unwrap());
            }

            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = query_row!(&conn, $query, &[&selector.to_string()], |x| {
                Ok(Self::$select_fn(x))
            });

            if res.is_err() {
                return Err(Error::GeneralNotFound($name_.to_string()));
            }

            let x = res.unwrap();
            self.2
                .set(
                    format!($cache_key_tmpl, selector),
                    serde_json::to_string(&x).unwrap(),
                )
                .await;

            Ok(x)
        }
    };

    ($name:ident($selector_t:ty as i64)@$select_fn:ident -> $query:literal --name=$name_:literal --returns=$returns_:tt --cache-key-tmpl=$cache_key_tmpl:literal) => {
        pub async fn $name(&self, selector: $selector_t) -> Result<$returns_> {
            if let Some(cached) = self
                .2
                .get(format!($cache_key_tmpl, selector.to_string()))
                .await
            {
                return Ok(serde_json::from_str(&cached).unwrap());
            }

            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = query_row!(&conn, $query, &[&(selector as i64)], |x| {
                Ok(Self::$select_fn(x))
            });

            if res.is_err() {
                return Err(Error::GeneralNotFound($name_.to_string()));
            }

            let x = res.unwrap();
            self.2
                .set(
                    format!($cache_key_tmpl, selector),
                    serde_json::to_string(&x).unwrap(),
                )
                .await;

            Ok(x)
        }
    };

    ($name:ident()@$select_fn:ident:$permission:ident -> $query:literal) => {
        pub async fn $name(&self, id: usize, user: User) -> Result<()> {
            let y = self.$select_fn(id).await?;

            if user.id != y.owner {
                if !user.permissions.check(FinePermission::$permission) {
                    return Err(Error::NotAllowed);
                } else {
                    self.create_audit_log_entry($crate::model::moderation::AuditLogEntry::new(
                        user.id,
                        format!("invoked `{}` with x value `{id}`", stringify!($name)),
                    ))
                }
            }

            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(&conn, $query, &[&(id as i64)]);

            if let Err(e) = res {
                return Err(Error::DatabaseError(e.to_string()));
            }

            Ok(())
        }
    };

    ($name:ident()@$select_fn:ident:$permission:ident -> $query:literal --cache-key-tmpl=$cache_key_tmpl:literal) => {
        pub async fn $name(&self, id: usize, user: User) -> Result<()> {
            let y = self.$select_fn(id).await?;

            if user.id != y.owner {
                if !user.permissions.check(FinePermission::$permission) {
                    return Err(Error::NotAllowed);
                } else {
                    self.create_audit_log_entry($crate::model::moderation::AuditLogEntry::new(
                        user.id,
                        format!("invoked `{}` with x value `{id}`", stringify!($name)),
                    ))
                }
            }

            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(&conn, $query, &[&(id as i64)]);

            if let Err(e) = res {
                return Err(Error::DatabaseError(e.to_string()));
            }

            self.2.remove(format!($cache_key_tmpl, id)).await;

            Ok(())
        }
    };

    ($name:ident($x:ty)@$select_fn:ident:$permission:ident -> $query:literal) => {
        pub async fn $name(&self, id: usize, user: User, x: $x) -> Result<()> {
            let y = self.$select_fn(id).await?;

            if user.id != y.owner {
                if !user.permissions.check(FinePermission::$permission) {
                    return Err(Error::NotAllowed);
                } else {
                    self.create_audit_log_entry($crate::model::moderation::AuditLogEntry::new(
                        user.id,
                        format!("invoked `{}` with x value `{id}`", stringify!($name)),
                    ))
                    .await?
                }
            }

            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(&conn, $query, &[&x, &(id as i64)]);

            if let Err(e) = res {
                return Err(Error::DatabaseError(e.to_string()));
            }

            Ok(())
        }
    };

    ($name:ident($x:ty)@$select_fn:ident:$permission:ident -> $query:literal --cache-key-tmpl=$cache_key_tmpl:literal) => {
        pub async fn $name(&self, id: usize, user: User, x: $x) -> Result<()> {
            let y = self.$select_fn(id).await?;

            if user.id != y.owner {
                if !user.permissions.check(FinePermission::$permission) {
                    return Err(Error::NotAllowed);
                } else {
                    self.create_audit_log_entry($crate::model::moderation::AuditLogEntry::new(
                        user.id,
                        format!("invoked `{}` with x value `{x}`", stringify!($name)),
                    ))
                    .await?
                }
            }

            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(&conn, $query, params![&x, &(id as i64)]);

            if let Err(e) = res {
                return Err(Error::DatabaseError(e.to_string()));
            }

            self.2.remove(format!($cache_key_tmpl, id)).await;

            Ok(())
        }
    };

    ($name:ident($x:ty)@$select_fn:ident:$permission:ident -> $query:literal --serde) => {
        pub async fn $name(&self, id: usize, user: User, x: $x) -> Result<()> {
            let y = self.$select_fn(id).await?;

            if user.id != y.owner {
                if !user.permissions.check(FinePermission::$permission) {
                    return Err(Error::NotAllowed);
                } else {
                    self.create_audit_log_entry($crate::model::moderation::AuditLogEntry::new(
                        user.id,
                        format!("invoked `{}` with x value `{id}`", stringify!($name), id),
                    ))
                    .await?
                }
            }

            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(
                &conn,
                $query,
                &[&serde_json::to_string(&x).unwrap(), &(id as i64)]
            );

            if let Err(e) = res {
                return Err(Error::DatabaseError(e.to_string()));
            }

            Ok(())
        }
    };

    ($name:ident($x:ty)@$select_fn:ident:$permission:ident -> $query:literal --serde --cache-key-tmpl=$cache_key_tmpl:literal) => {
        pub async fn $name(&self, id: usize, user: User, x: $x) -> Result<()> {
            let y = self.$select_fn(id).await?;

            if user.id != y.owner {
                if !user.permissions.check(FinePermission::$permission) {
                    return Err(Error::NotAllowed);
                } else {
                    self.create_audit_log_entry($crate::model::moderation::AuditLogEntry::new(
                        user.id,
                        format!("invoked `{}` with x value `{x:?}`", stringify!($name)),
                    ))
                    .await?
                }
            }

            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(
                &conn,
                $query,
                &[&serde_json::to_string(&x).unwrap(), &(id as i64)]
            );

            if let Err(e) = res {
                return Err(Error::DatabaseError(e.to_string()));
            }

            self.2.remove(format!($cache_key_tmpl, id)).await;

            Ok(())
        }
    };

    ($name:ident($x:ty) -> $query:literal) => {
        pub async fn $name(&self, id: usize, x: $x) -> Result<()> {
            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(&conn, $query, &[&x, &(id as i64)]);

            if let Err(e) = res {
                return Err(Error::DatabaseError(e.to_string()));
            }

            Ok(())
        }
    };

    ($name:ident($x:ty) -> $query:literal --cache-key-tmpl=$cache_key_tmpl:literal) => {
        pub async fn $name(&self, id: usize, x: $x) -> Result<()> {
            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(&conn, $query, &[&x, &(id as i64)]);

            if let Err(e) = res {
                return Err(Error::DatabaseError(e.to_string()));
            }

            self.2.remove(format!($cache_key_tmpl, id)).await;

            Ok(())
        }
    };

    ($name:ident($x:ty) -> $query:literal --serde) => {
        pub async fn $name(&self, id: usize, x: $x) -> Result<()> {
            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(
                &conn,
                $query,
                &[&serde_json::to_string(&x).unwrap(), &(id as i64)]
            );

            if let Err(e) = res {
                return Err(Error::DatabaseError(e.to_string()));
            }

            Ok(())
        }
    };

    ($name:ident($x:ty) -> $query:literal --serde --cache-key-tmpl=$cache_key_tmpl:literal) => {
        pub async fn $name(&self, id: usize, x: $x) -> Result<()> {
            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(
                &conn,
                $query,
                &[&serde_json::to_string(&x).unwrap(), &(id as i64)]
            );

            if let Err(e) = res {
                return Err(Error::DatabaseError(e.to_string()));
            }

            self.2.remove(format!($cache_key_tmpl, id)).await;

            Ok(())
        }
    };

    ($name:ident() -> $query:literal --cache-key-tmpl=$cache_key_tmpl:literal --incr) => {
        pub async fn $name(&self, id: usize) -> Result<()> {
            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(&conn, $query, &[&(id as i64)]);

            if let Err(e) = res {
                return Err(Error::DatabaseError(e.to_string()));
            }

            self.2.remove(format!($cache_key_tmpl, id)).await;

            Ok(())
        }
    };

    ($name:ident() -> $query:literal --cache-key-tmpl=$cache_key_tmpl:literal --decr) => {
        pub async fn $name(&self, id: usize) -> Result<()> {
            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(&conn, $query, &[&(id as i64)]);

            if let Err(e) = res {
                return Err(Error::DatabaseError(e.to_string()));
            }

            self.2.remove(format!($cache_key_tmpl, id)).await;

            Ok(())
        }
    };

    ($name:ident()@$select_fn:ident:$permission:ident -> $query:literal --cache-key-tmpl=$cache_key_tmpl:ident) => {
        pub async fn $name(&self, id: usize, user: User) -> Result<()> {
            let y = self.$select_fn(id).await?;

            if user.id != y.owner {
                if !user.permissions.check(FinePermission::$permission) {
                    return Err(Error::NotAllowed);
                } else {
                    self.create_audit_log_entry($crate::model::moderation::AuditLogEntry::new(
                        user.id,
                        format!("invoked `{}` with x value `{id}`", stringify!($name)),
                    ))
                    .await?
                }
            }

            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(&conn, $query, &[&(id as i64)]);

            if let Err(e) = res {
                return Err(Error::DatabaseError(e.to_string()));
            }

            self.$cache_key_tmpl(&y).await;

            Ok(())
        }
    };

    ($name:ident($x:ty)@$select_fn:ident:$permission:ident -> $query:literal --cache-key-tmpl=$cache_key_tmpl:ident) => {
        pub async fn $name(&self, id: usize, user: User, x: $x) -> Result<()> {
            let y = self.$select_fn(id).await?;

            if user.id != y.owner {
                if !user.permissions.check(FinePermission::$permission) {
                    return Err(Error::NotAllowed);
                } else {
                    self.create_audit_log_entry(crate::model::moderation::AuditLogEntry::new(
                        user.id,
                        format!("invoked `{}` with x value `{x}`", stringify!($name)),
                    ))
                    .await?
                }
            }

            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(&conn, $query, params![&x, &(id as i64)]);

            if let Err(e) = res {
                return Err(Error::DatabaseError(e.to_string()));
            }

            self.$cache_key_tmpl(&y).await;

            Ok(())
        }
    };

    ($name:ident($x:ty)@$select_fn:ident -> $query:literal --cache-key-tmpl=$cache_key_tmpl:ident) => {
        pub async fn $name(&self, id: usize, x: $x) -> Result<()> {
            let y = self.$select_fn(id).await?;

            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(&conn, $query, params![&x, &(id as i64)]);

            if let Err(e) = res {
                return Err(Error::DatabaseError(e.to_string()));
            }

            self.$cache_key_tmpl(&y).await;

            Ok(())
        }
    };

    ($name:ident($x:ty)@$select_fn:ident -> $query:literal --serde --cache-key-tmpl=$cache_key_tmpl:ident) => {
        pub async fn $name(&self, id: usize, x: $x) -> Result<()> {
            let y = self.$select_fn(id).await?;

            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(
                &conn,
                $query,
                params![&serde_json::to_string(&x).unwrap(), &(id as i64)]
            );

            if let Err(e) = res {
                return Err(Error::DatabaseError(e.to_string()));
            }

            self.$cache_key_tmpl(&y).await;

            Ok(())
        }
    };

    ($name:ident($x:ty)@$select_fn:ident:$permission:ident -> $query:literal --serde --cache-key-tmpl=$cache_key_tmpl:ident) => {
        pub async fn $name(&self, id: usize, user: User, x: $x) -> Result<()> {
            let y = self.$select_fn(id).await?;

            if user.id != y.owner {
                if !user.permissions.check(FinePermission::$permission) {
                    return Err(Error::NotAllowed);
                } else {
                    self.create_audit_log_entry(crate::model::moderation::AuditLogEntry::new(
                        user.id,
                        format!("invoked `{}` with x value `{x:?}`", stringify!($name)),
                    ))
                    .await?
                }
            }

            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(
                &conn,
                $query,
                params![&serde_json::to_string(&x).unwrap(), &(id as i64)]
            );

            if let Err(e) = res {
                return Err(Error::DatabaseError(e.to_string()));
            }

            self.$cache_key_tmpl(&y).await;

            Ok(())
        }
    };

    ($name:ident()@$select_fn:ident -> $query:literal --cache-key-tmpl=$cache_key_tmpl:ident --incr) => {
        pub async fn $name(&self, id: usize) -> Result<()> {
            let y = self.$select_fn(id).await?;

            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(&conn, $query, &[&(id as i64)]);

            if let Err(e) = res {
                return Err(Error::DatabaseError(e.to_string()));
            }

            self.$cache_key_tmpl(&y).await;

            Ok(())
        }
    };

    ($name:ident()@$select_fn:ident -> $query:literal --cache-key-tmpl=$cache_key_tmpl:ident --decr) => {
        pub async fn $name(&self, id: usize) -> Result<()> {
            let y = self.$select_fn(id).await?;

            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(&conn, $query, &[&(id as i64)]);

            if let Err(e) = res {
                return Err(Error::DatabaseError(e.to_string()));
            }

            self.$cache_key_tmpl(&y).await;

            Ok(())
        }
    };
}
