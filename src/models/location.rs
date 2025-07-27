use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Location {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateLocation {
    pub name: String,
}