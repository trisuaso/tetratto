use crate::config::Config;
use rusqlite::{Connection, Result};

#[derive(Clone)]
pub struct DataManager(pub Config);

impl DataManager {
    /// Obtain a connection to the staging database.
    pub(crate) async fn connect(&self) -> Result<Connection> {
        Ok(Connection::open(&self.0.database.name)?)
    }

    /// Create a new [`DataManager`] (and init database).
    pub async fn new(config: Config) -> Result<Self> {
        let this = Self(config.clone());
        let conn = this.connect().await?;

        conn.pragma_update(None, "journal_mode", "WAL").unwrap();
        conn.execute(super::common::CREATE_TABLE_USERS, ()).unwrap();

        Ok(this)
    }
}

#[cfg(feature = "sqlite")]
#[macro_export]
macro_rules! get {
    ($row:ident->$idx:literal($t:tt)) => {
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
macro_rules! execute {
    ($conn:expr, $sql:expr, $params:expr) => {
        $conn.prepare($sql).unwrap().execute($params)
    };
}
