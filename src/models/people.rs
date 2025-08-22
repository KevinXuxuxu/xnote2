use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct People {
    pub id: i32,
    pub name: String,
    pub notes: Option<String>,
}
