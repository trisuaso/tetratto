#[cfg(not(feature = "redis"))]
use crate::cache::no_cache::NoCache;
#[cfg(feature = "redis")]
use crate::cache::redis::RedisCache;

use crate::cache::Cache;

use crate::config::Config;
use bb8_postgres::{
    PostgresConnectionManager,
    bb8::{Pool, PooledConnection},
};
use tetratto_l10n::{LangFile, read_langs};
use tokio_postgres::{Config as PgConfig, NoTls, Row, types::ToSql};

pub type Result<T> = std::result::Result<T, tokio_postgres::Error>;
pub type Connection<'a> = PooledConnection<'a, PostgresConnectionManager<NoTls>>;

#[derive(Clone)]
pub struct DataManager(
    pub Config,
    pub HashMap<String, LangFile>,
    #[cfg(feature = "redis")] pub RedisCache,
    #[cfg(not(feature = "redis"))] pub NoCache,
    pub Pool<PostgresConnectionManager<NoTls>>,
);

impl DataManager {
    /// Obtain a connection to the staging database.
    pub(crate) async fn connect(&self) -> Result<Connection> {
        Ok(self.3.get().await.unwrap())
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
        Ok(Self(
            config.clone(),
            read_langs(),
            #[cfg(feature = "redis")]
            RedisCache::new().await,
            #[cfg(not(feature = "redis"))]
            NoCache::new().await,
            pool,
        ))
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
