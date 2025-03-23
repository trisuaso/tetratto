use crate::config::Config;
use bb8_postgres::{
    PostgresConnectionManager,
    bb8::{Pool, PooledConnection},
};
use tokio_postgres::{Config as PgConfig, NoTls, Row, types::ToSql};

pub type Result<T> = std::result::Result<T, tokio_postgres::Error>;
pub type Connection<'a> = PooledConnection<'a, PostgresConnectionManager<NoTls>>;

#[derive(Clone)]
pub struct DataManager(pub Config, pub Pool<PostgresConnectionManager<NoTls>>);

impl DataManager {
    /// Obtain a connection to the staging database.
    pub(crate) async fn connect(&self) -> Result<Connection> {
        Ok(self.1.get().await.unwrap())
    }

    /// Create a new [`DataManager`] (and init database).
    pub async fn new(config: Config) -> Result<Self> {
        let manager = PostgresConnectionManager::new(
            {
                let mut c = PgConfig::new();
                c.user(&config.database.user);
                c.password(&config.database.password);
                c.dbname(&config.database.name);
                c
            },
            NoTls,
        );
        let pool = Pool::builder().max_size(15).build(manager).await.unwrap();

        let this = Self(config.clone(), pool);
        let c = this.clone();
        let conn = c.connect().await?;

        conn.execute(super::common::CREATE_TABLE_USERS, &[])
            .await
            .unwrap();

        Ok(this)
    }
}

#[cfg(feature = "postgres")]
#[macro_export]
macro_rules! get {
    ($row:ident->$idx:literal($t:tt)) => {
        $row.get::<usize, Option<$t>>($idx).unwrap()
    };
}

pub async fn query_row_helper<T, F>(
    conn: &Connection<'_>,
    sql: &str,
    params: &[&(dyn ToSql + Sync)],
    f: F,
) -> Result<T>
where
    F: FnOnce(&Row) -> Result<T>,
{
    let query = conn.prepare(sql).await.unwrap();
    let res = conn.query_one(&query, params).await;

    if let Ok(row) = res {
        Ok(f(&row).unwrap())
    } else {
        Err(res.unwrap_err())
    }
}

#[macro_export]
macro_rules! query_row {
    ($conn:expr, $sql:expr, $params:expr, $f:expr) => {
        crate::database::query_row_helper($conn, $sql, $params, $f).await
    };
}

pub async fn execute_helper(
    conn: &Connection<'_>,
    sql: &str,
    params: &[&(dyn ToSql + Sync)],
) -> Result<()> {
    let query = conn.prepare(sql).await.unwrap();
    conn.execute(&query, params).await?;
    Ok(())
}

#[macro_export]
macro_rules! execute {
    ($conn:expr, $sql:expr, $params:expr) => {
        crate::database::execute_helper($conn, $sql, $params).await
    };
}
