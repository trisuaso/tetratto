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

        execute!(&conn, common::CREATE_TABLE_USERS, []).unwrap();
        execute!(&conn, common::CREATE_TABLE_PAGES, []).unwrap();

        Ok(())
    }
}
