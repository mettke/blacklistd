use diesel::sql_types::{BigInt, Timestamp};
use serde::Serialize;

#[derive(Debug, Clone, QueryableByName, Serialize)]
pub struct IpsPerTime {
    #[sql_type = "BigInt"]
    pub count: i64,
    #[sql_type = "Timestamp"]
    pub last_update_start: chrono::NaiveDateTime,
    #[sql_type = "Timestamp"]
    pub last_update_end: chrono::NaiveDateTime,
}
