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
        execute!(&conn, common::CREATE_TABLE_PAGES).unwrap();
        execute!(&conn, common::CREATE_TABLE_ENTRIES).unwrap();
        execute!(&conn, common::CREATE_TABLE_MEMBERSHIPS).unwrap();
        execute!(&conn, common::CREATE_TABLE_REACTIONS).unwrap();
        execute!(&conn, common::CREATE_TABLE_NOTIFICATIONS).unwrap();
        execute!(&conn, common::CREATE_TABLE_USERFOLLOWS).unwrap();
        execute!(&conn, common::CREATE_TABLE_USERBLOCKS).unwrap();
        execute!(&conn, common::CREATE_TABLE_IPBANS).unwrap();

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
            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = query_row!(&conn, $query, &[&selector], |x| { Ok(Self::$select_fn(x)) });

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
                }
            }

            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(&conn, $query, &[&id.to_string()]);

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
                }
            }

            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(&conn, $query, &[&id.to_string()]);

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
                }
            }

            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(&conn, $query, &[&x, &id.to_string()]);

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
                }
            }

            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(&conn, $query, &[&x, &id.to_string()]);

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
                }
            }

            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(
                &conn,
                $query,
                &[&serde_json::to_string(&x).unwrap(), &id.to_string()]
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
                }
            }

            let conn = match self.connect().await {
                Ok(c) => c,
                Err(e) => return Err(Error::DatabaseConnection(e.to_string())),
            };

            let res = execute!(
                &conn,
                $query,
                &[&serde_json::to_string(&x).unwrap(), &id.to_string()]
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

            let res = execute!(&conn, $query, &[&x, &id.to_string()]);

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

            let res = execute!(&conn, $query, &[&x, &id.to_string()]);

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
                &[&serde_json::to_string(&x).unwrap(), &id.to_string()]
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
                &[&serde_json::to_string(&x).unwrap(), &id.to_string()]
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

            let res = execute!(&conn, $query, &[&id.to_string()]);

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

            let res = execute!(&conn, $query, &[&id.to_string()]);

            if let Err(e) = res {
                return Err(Error::DatabaseError(e.to_string()));
            }

            self.2.remove(format!($cache_key_tmpl, id)).await;

            Ok(())
        }
    };
}
