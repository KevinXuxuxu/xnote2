use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ActivityType {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Activity {
    pub id: i32,
    pub name: String,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub activity_type: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateActivity {
    pub name: String,
    #[serde(rename = "type")]
    pub activity_type: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateActivity {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub activity_type: Option<String>,
}
