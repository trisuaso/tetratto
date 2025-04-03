#[cfg(not(feature = "redis"))]
use crate::cache::no_cache::NoCache;
#[cfg(feature = "redis")]
use crate::cache::redis::RedisCache;

use crate::cache::Cache;

use crate::config::Config;
use rusqlite::{Connection, Result};
use std::collections::HashMap;
use tetratto_l10n::{LangFile, read_langs};

#[derive(Clone)]
pub struct DataManager(
    pub Config,
    pub HashMap<String, LangFile>,
    #[cfg(feature = "redis")] pub RedisCache,
    #[cfg(not(feature = "redis"))] pub NoCache,
);

impl DataManager {
    /// Obtain a connection to the staging database.
    pub(crate) async fn connect(&self) -> Result<Connection> {
        Ok(Connection::open(&self.0.database.name)?)
    }

    /// Create a new [`DataManager`] (and init database).
    pub async fn new(config: Config) -> Result<Self> {
        let this = Self(
            config.clone(),
            read_langs(),
            #[cfg(feature = "redis")]
            RedisCache::new().await,
            #[cfg(not(feature = "redis"))]
            NoCache::new().await,
        );

        let conn = this.connect().await?;
        conn.pragma_update(None, "journal_mode", "WAL").unwrap();

        Ok(this)
    }
}

#[macro_export]
macro_rules! get {
    ($row:ident->$idx:literal($t:ty)) => {
        $row.get::<usize, $t>($idx).unwrap()
    };
}

#[macro_export]
macro_rules! query_row {
    ($conn:expr, $sql:expr, $params:expr, $f:expr) => {{
        let mut query = $conn.prepare($sql).unwrap();
        query.query_row($params, $f)
    }};
}

#[macro_export]
macro_rules! query_rows {
    ($conn:expr, $sql:expr, $params:expr, $f:expr) => {{
        let mut query = $conn.prepare($sql).unwrap();

        if let Ok(mut rows) = query.query($params) {
            let mut out = Vec::new();

            while let Some(row) = rows.next().unwrap() {
                out.push($f(&row));
            }

            Ok(out)
        } else {
            Err(Error::Unknown)
        }
    }};
}

#[macro_export]
macro_rules! execute {
    ($conn:expr, $sql:expr, $params:expr) => {
        $conn.prepare($sql).unwrap().execute($params)
    };

    ($conn:expr, $sql:expr) => {
        $conn.prepare($sql).unwrap().execute(())
    };
}

#[macro_export]
macro_rules! params {
    () => {
        rusqlite::params![]
    };
    ($($params:expr),+ $(,)?) => {
        rusqlite::params![$($params),+]
    };
}
