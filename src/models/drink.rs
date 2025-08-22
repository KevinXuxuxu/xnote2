use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DrinkOption {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Drink {
    pub id: i32,
    pub name: String,
    pub date: NaiveDate,
}

#[derive(Debug, Deserialize)]
pub struct CreateDrink {
    pub date: NaiveDate,
    pub name: String,
    pub people_ids: Vec<i32>,
}

#[derive(Debug, Serialize)]
pub struct CreateDrinkResponse {
    pub id: i32,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DrinkPeople {
    pub drink: i32,
    pub people: i32,
}
