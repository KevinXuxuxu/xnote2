use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Meal {
    pub id: i32,
    pub date: NaiveDate,
    pub time: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMeal {
    pub date: NaiveDate,
    pub time: String,
    pub notes: Option<String>,
    pub food_source: CreateMealFoodSource,
    pub people_ids: Vec<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum CreateMealFoodSource {
    #[serde(rename = "recipe")]
    Recipe { recipe_id: i32, meal_type: String },
    #[serde(rename = "product")]
    Product { product_id: i32, meal_type: String },
    #[serde(rename = "restaurant")]
    Restaurant {
        restaurant_id: i32,
        meal_type: String,
    },
}

#[derive(Debug, Serialize)]
pub struct CreateMealResponse {
    pub id: i32,
    pub date: NaiveDate,
    pub time: String,
    pub notes: Option<String>,
}
