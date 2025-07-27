use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct FoodType {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Restaurant {
    pub id: i32,
    pub name: String,
    pub location: String,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub food_type: String,
    pub price: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRestaurant {
    pub name: String,
    pub location: String,
    #[serde(rename = "type")]
    pub food_type: String,
    pub price: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRestaurant {
    pub name: Option<String>,
    pub location: Option<String>,
    #[serde(rename = "type")]
    pub food_type: Option<String>,
    pub price: Option<f32>,
}