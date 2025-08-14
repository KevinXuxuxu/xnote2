use actix_web::{web, HttpResponse, Result};
use chrono::{NaiveDate, Datelike, Weekday};
use sqlx::PgPool;
use crate::models::daily_summary::{DailySummary, DailySummaryQuery};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/daily-summary")
            .route(web::get().to(get_daily_summary))
    );
}

async fn get_daily_summary(
    pool: web::Data<PgPool>, 
    query: web::Query<DailySummaryQuery>
) -> Result<HttpResponse> {
    let start_date = query.start_date.unwrap_or_else(|| {
        chrono::Utc::now().naive_utc().date() - chrono::Duration::days(30)
    });
    
    let end_date = query.end_date.unwrap_or_else(|| {
        chrono::Utc::now().naive_utc().date()
    });

    match build_daily_summaries(&pool, start_date, end_date).await {
        Ok(summaries) => Ok(HttpResponse::Ok().json(summaries)),
        Err(e) => {
            log::error!("Failed to fetch daily summaries: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch daily summaries"
            })))
        }
    }
}

async fn build_daily_summaries(
    pool: &PgPool,
    start_date: NaiveDate,
    end_date: NaiveDate
) -> Result<Vec<DailySummary>, sqlx::Error> {
    let mut summaries = Vec::new();
    
    // Generate all dates in range
    let mut current_date = start_date;
    while current_date <= end_date {
        let summary = build_summary_for_date(pool, current_date).await?;
        summaries.push(summary);
        current_date += chrono::Duration::days(1);
    }
    
    Ok(summaries)
}

async fn build_summary_for_date(
    pool: &PgPool,
    date: NaiveDate
) -> Result<DailySummary, sqlx::Error> {
    // Get meals for the date
    let meals = get_meals_for_date(pool, date).await?;
    
    // Get events for the date  
    let events = get_events_for_date(pool, date).await?;
    
    // Get drinks for the date
    let drinks = get_drinks_for_date(pool, date).await?;
    
    // Build summary
    let day_of_week = match date.weekday() {
        Weekday::Mon => "Monday",
        Weekday::Tue => "Tuesday", 
        Weekday::Wed => "Wednesday",
        Weekday::Thu => "Thursday",
        Weekday::Fri => "Friday",
        Weekday::Sat => "Saturday",
        Weekday::Sun => "Sunday",
    }.to_string();
    
    // Organize meals by time
    let mut breakfast = [String::new(), String::new()];
    let mut lunch = [String::new(), String::new()];
    let mut dinner = [String::new(), String::new()];
    
    let mut breakfast_count = 0;
    let mut lunch_count = 0;
    let mut dinner_count = 0;
    
    for meal in meals {
        let meal_summary = format_meal_summary(&meal);
        
        match meal.time.as_str() {
            "breakfast" if breakfast_count < 2 => {
                breakfast[breakfast_count] = meal_summary;
                breakfast_count += 1;
            },
            "lunch" if lunch_count < 2 => {
                lunch[lunch_count] = meal_summary;
                lunch_count += 1;
            },
            "dinner" if dinner_count < 2 => {
                dinner[dinner_count] = meal_summary;
                dinner_count += 1;
            },
            _ => {} // Skip if we have too many meals of this type
        }
    }
    
    // Format events (limit to 10)
    let mut event_array = std::array::from_fn(|_| String::new());
    for (i, event) in events.iter().take(10).enumerate() {
        event_array[i] = format_event_summary(event);
    }
    
    // Format drinks
    let drink_summaries = drinks.iter().map(|d| format_drink_summary(d)).collect();
    
    Ok(DailySummary {
        date,
        day_of_week,
        breakfast,
        lunch,
        dinner,
        drinks: drink_summaries,
        events: event_array,
    })
}

#[derive(Debug)]
struct MealRow {
    time: String,
    notes: Option<String>,
    food_source_name: Option<String>,
    meal_type: Option<String>,
    people_names: Vec<String>,
}

async fn get_meals_for_date(pool: &PgPool, date: NaiveDate) -> Result<Vec<MealRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        WITH meal_food_source AS (
            SELECT 
                m.id as meal_id,
                m.date,
                m."time",
                m.notes,
                CASE 
                    WHEN mr.meal IS NOT NULL THEN r.name
                    WHEN mp.meal IS NOT NULL THEN p.name  
                    WHEN mrt.meal IS NOT NULL THEN rt.name
                END as food_source_name,
                COALESCE(mr.type, mp.type, mrt.type) as meal_type
            FROM meal m
            LEFT JOIN meal_recipe mr ON m.id = mr.meal
            LEFT JOIN recipe r ON mr.recipe = r.id
            LEFT JOIN meal_product mp ON m.id = mp.meal
            LEFT JOIN product p ON mp.product = p.id
            LEFT JOIN meal_restaurant mrt ON m.id = mrt.meal
            LEFT JOIN restaurant rt ON mrt.restaurant = rt.id
            WHERE m.date = $1
        ),
        meal_people_agg AS (
            SELECT 
                mp.meal,
                array_agg(pe.name ORDER BY pe.name) as people_names
            FROM meal_people mp
            JOIN people pe ON mp.people = pe.id
            GROUP BY mp.meal
        )
        SELECT 
            mfs.meal_id,
            mfs."time",
            mfs.notes,
            mfs.food_source_name,
            mfs.meal_type,
            COALESCE(mpa.people_names, ARRAY[]::text[]) as "people_names!"
        FROM meal_food_source mfs
        LEFT JOIN meal_people_agg mpa ON mfs.meal_id = mpa.meal
        ORDER BY mfs."time", mfs.meal_id
        "#,
        date
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|row| MealRow {
        time: row.time,
        notes: row.notes,
        food_source_name: row.food_source_name,
        meal_type: row.meal_type,
        people_names: row.people_names,
    }).collect())
}

#[derive(Debug)]
struct EventRow {
    activity_name: String,
    measure: Option<String>,
    location: Option<String>,
    notes: Option<String>,
    people_names: Vec<String>,
}

async fn get_events_for_date(pool: &PgPool, date: NaiveDate) -> Result<Vec<EventRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        WITH event_people_agg AS (
            SELECT 
                ep.event,
                array_agg(pe.name ORDER BY pe.name) as people_names
            FROM event_people ep
            JOIN people pe ON ep.people = pe.id
            GROUP BY ep.event
        )
        SELECT 
            e.id,
            a.name as activity_name,
            e.measure,
            e.location,
            e.notes,
            COALESCE(epa.people_names, ARRAY[]::text[]) as "people_names!"
        FROM event e
        JOIN activity a ON e.activity = a.id
        LEFT JOIN event_people_agg epa ON e.id = epa.event
        WHERE e.date = $1
        ORDER BY e.id
        "#,
        date
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|row| EventRow {
        activity_name: row.activity_name,
        measure: row.measure,
        location: row.location,
        notes: row.notes,
        people_names: row.people_names,
    }).collect())
}

#[derive(Debug)]
struct DrinkRow {
    name: String,
    people_names: Vec<String>,
}

async fn get_drinks_for_date(pool: &PgPool, date: NaiveDate) -> Result<Vec<DrinkRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        WITH drink_people_agg AS (
            SELECT 
                dp.drink,
                array_agg(pe.name ORDER BY pe.name) as people_names
            FROM drink_people dp
            JOIN people pe ON dp.people = pe.id
            GROUP BY dp.drink
        )
        SELECT 
            d.id,
            d.name,
            COALESCE(dpa.people_names, ARRAY[]::text[]) as "people_names!"
        FROM drink d
        LEFT JOIN drink_people_agg dpa ON d.id = dpa.drink
        WHERE d.date = $1
        ORDER BY d.id
        "#,
        date
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|row| DrinkRow {
        name: row.name,
        people_names: row.people_names,
    }).collect())
}

fn format_meal_summary(meal: &MealRow) -> String {
    let mut parts = Vec::new();
    
    // Add food source name
    if let Some(name) = &meal.food_source_name {
        parts.push(name.clone());
    }
    
    // Add meal type if meaningful
    if let Some(meal_type) = &meal.meal_type {
        if meal_type != "cooked" { // Only show non-default types
            parts.push(format!("({})", meal_type));
        }
    }
    
    // Add people with smart formatting
    if !meal.people_names.is_empty() {
        parts.push(format!("with {}", meal.people_names.join(", ")));
    }
    
    // Add notes if present
    if let Some(notes) = &meal.notes {
        if !notes.trim().is_empty() {
            parts.push(format!("- {}", notes.trim()));
        }
    }
    
    parts.join(" ")
}

fn format_event_summary(event: &EventRow) -> String {
    let mut parts = Vec::new();
    
    // Start with activity name
    parts.push(event.activity_name.clone());
    
    // Add location with @ prefix if present
    if let Some(location) = &event.location {
        if !location.trim().is_empty() {
            parts.push(format!("@{}", location.trim()));
        }
    }
    
    // Add measure with "for" prefix if present
    if let Some(measure) = &event.measure {
        if !measure.trim().is_empty() {
            parts.push(format!("for {}", measure.trim()));
        }
    }
    
    // Add people with "with" prefix if present
    if !event.people_names.is_empty() {
        parts.push(format!("with {}", event.people_names.join(", ")));
    }
    
    // Add notes if present
    if let Some(notes) = &event.notes {
        if !notes.trim().is_empty() {
            parts.push(format!("- {}", notes.trim()));
        }
    }
    
    parts.join(" ")
}

fn format_drink_summary(drink: &DrinkRow) -> String {
    let mut parts = Vec::new();
    
    // Start with drink name
    parts.push(drink.name.clone());
    
    // Add people if present
    if !drink.people_names.is_empty() {
        parts.push(format!("with {}", drink.people_names.join(", ")));
    }
    
    parts.join(" ")
}