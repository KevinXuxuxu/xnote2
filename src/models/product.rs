use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProduct {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProduct {
    pub name: Option<String>,
}
