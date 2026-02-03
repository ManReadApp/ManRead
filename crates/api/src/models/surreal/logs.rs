use serde::{Deserialize, Serialize};
use surrealdb::Datetime;
use surrealdb_extras::{SurrealTable, SurrealTableInfo};

use crate::{
    error::{ApiError, ApiResult},
    init::db::DB,
    models::tag::Empty,
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

#[derive(Default)]
pub struct LogDbService;

impl LogDbService {
    pub async fn error(err: ApiError) -> ApiResult<()> {
        LogMessage {
            message: err.to_string(),
            level: LogLevel::Error,
            created_at: Datetime::default(),
        }
        .add(&*DB)
        .await?;
        Ok(())
    }

    pub async fn list(&self) -> ApiResult<Vec<LogMessage>> {
        let v: Vec<LogMessage> = LogMessage::all(&*DB).await?;
        Ok(v)
    }
    pub async fn clear(&self) -> ApiResult<()> {
        let _: Vec<Empty> = DB.delete(LogMessage::name()).await?;
        Ok(())
    }
}
