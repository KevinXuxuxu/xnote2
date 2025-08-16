use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MealItem {
    pub name: String,          // Recipe/product/restaurant name
    pub people: String,        // Comma-separated people names
    pub notes: Option<String>, // Notes if any
    #[serde(rename = "type")]
    pub meal_type: String,     // cooked, dine-in, etc.
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventItem {
    pub text: String,
    #[serde(rename = "type")]
    pub activity_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailySummary {
    pub date: NaiveDate,
    pub day_of_week: String,
    pub breakfast: Vec<MealItem>,
    pub lunch: Vec<MealItem>, 
    pub dinner: Vec<MealItem>,
    pub drinks: Vec<String>,
    pub events: Vec<EventItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailySummaryQuery {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}