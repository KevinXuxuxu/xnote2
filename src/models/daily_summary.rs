use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MealItem {
    pub text: String,
    #[serde(rename = "type")]
    pub meal_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailySummary {
    pub date: NaiveDate,
    pub day_of_week: String,
    pub breakfast: Vec<MealItem>,
    pub lunch: Vec<MealItem>, 
    pub dinner: Vec<MealItem>,
    pub drinks: Vec<String>,
    pub events: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailySummaryQuery {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}