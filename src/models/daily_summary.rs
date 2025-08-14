use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DailySummary {
    pub date: NaiveDate,
    pub day_of_week: String,
    pub breakfast: [String; 2],
    pub lunch: [String; 2], 
    pub dinner: [String; 2],
    pub drinks: Vec<String>,
    pub events: [String; 10],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailySummaryQuery {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}