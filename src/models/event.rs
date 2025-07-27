use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub id: i32,
    pub date: NaiveDate,
    pub activity: i32,
    pub measure: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEvent {
    pub date: NaiveDate,
    pub activity: i32,
    pub measure: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct EventPeople {
    pub event: i32,
    pub people: i32,
}