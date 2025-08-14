use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DailySummary {
    pub date: NaiveDate,
    pub day_of_week: String,
    pub breakfast: Vec<String>,
    pub lunch: Vec<String>, 
    pub dinner: Vec<String>,
    pub drinks: Vec<String>,
    pub events: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailySummaryQuery {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}