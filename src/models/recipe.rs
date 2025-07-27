use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Recipe {
    pub id: i32,
    pub name: String,
    pub ingredients: String,
    pub procedure: String,
    pub cautions: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRecipe {
    pub name: String,
    pub ingredients: String,
    pub procedure: String,
    pub cautions: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRecipe {
    pub name: Option<String>,
    pub ingredients: Option<String>,
    pub procedure: Option<String>,
    pub cautions: Option<String>,
}