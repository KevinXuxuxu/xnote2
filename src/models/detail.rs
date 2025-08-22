use crate::models::{people::People, product::Product, recipe::Recipe, restaurant::Restaurant};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize)]
pub struct MealDetail {
    pub id: i32,
    pub date: NaiveDate,
    pub time: String,
    pub notes: Option<String>,
    pub food_source: Option<MealFoodSource>,
    pub people: Vec<People>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum MealFoodSource {
    #[serde(rename = "recipe")]
    Recipe { recipe: Recipe, meal_type: String },
    #[serde(rename = "product")]
    Product { product: Product, meal_type: String },
    #[serde(rename = "restaurant")]
    Restaurant {
        restaurant: Restaurant,
        meal_type: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventDetail {
    pub id: i32,
    pub date: NaiveDate,
    pub activity: ActivityDetail,
    pub measure: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
    pub people: Vec<People>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ActivityDetail {
    pub id: i32,
    pub name: String,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub activity_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DrinkDetail {
    pub id: i32,
    pub name: String,
    pub date: NaiveDate,
    pub people: Vec<People>,
}
