use serde::{Deserialize, Serialize};
use surrealdb::Datetime;
use surrealdb_extras::{SurrealTable, SurrealTableInfo};

use crate::{
    error::{DbError, DbResult},
    tag::Empty,
    DbSession,
};

#[derive(SurrealTable, Serialize, Deserialize, Debug, Clone)]
#[db("errors")]
pub struct LogMessage {
    pub message: String,
    pub level: LogLevel,
    pub created_at: Datetime,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Clone)]
pub struct LogDbService {
    db: DbSession,
}

impl Default for LogDbService {
    fn default() -> Self {
        Self::new(crate::global_db())
    }
}

impl LogDbService {
    pub fn new(db: DbSession) -> Self {
        Self { db }
    }

    pub async fn error(&self, err: DbError) -> DbResult<()> {
        LogMessage {
            message: err.to_string(),
            level: LogLevel::Error,
            created_at: Datetime::default(),
        }
        .add(self.db.as_ref())
        .await?;
        Ok(())
    }

    pub async fn list(&self) -> DbResult<Vec<LogMessage>> {
        let v: Vec<LogMessage> = LogMessage::all(self.db.as_ref()).await?;
        Ok(v)
    }
    pub async fn clear(&self) -> DbResult<()> {
        let _: Vec<Empty> = self.db.delete(LogMessage::name()).await?;
        Ok(())
    }
}
