use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct MealTime {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct MealType {
    pub name: String,
}

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
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct MealRecipe {
    pub meal: i32,
    pub recipe: i32,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub meal_type: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct MealProduct {
    pub meal: i32,
    pub product: i32,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub meal_type: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct MealRestaurant {
    pub meal: i32,
    pub restaurant: i32,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub meal_type: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct MealPeople {
    pub meal: i32,
    pub people: i32,
}