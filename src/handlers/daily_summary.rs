use actix_web::{web, HttpResponse, Result};
use chrono::NaiveDate;
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
    let summaries = sqlx::query!(
        r#"
        WITH date_range AS (
            SELECT generate_series($1::date, $2::date, '1 day'::interval)::date AS date
        ),
        meal_aggregated AS (
            SELECT 
                m.date,
                m."time" as meal_time,
                CASE 
                    WHEN mr.meal IS NOT NULL THEN r.name
                    WHEN mp.meal IS NOT NULL THEN p.name  
                    WHEN mrt.meal IS NOT NULL THEN rt.name
                END as food_source_name,
                COALESCE(mr.type, mp.type, mrt.type) as meal_type,
                m.notes,
                array_agg(pe.name ORDER BY pe.name) FILTER (WHERE pe.name IS NOT NULL) as people_names
            FROM meal m
            LEFT JOIN meal_recipe mr ON m.id = mr.meal
            LEFT JOIN recipe r ON mr.recipe = r.id
            LEFT JOIN meal_product mp ON m.id = mp.meal
            LEFT JOIN product p ON mp.product = p.id
            LEFT JOIN meal_restaurant mrt ON m.id = mrt.meal
            LEFT JOIN restaurant rt ON mrt.restaurant = rt.id
            LEFT JOIN meal_people mpe ON m.id = mpe.meal
            LEFT JOIN people pe ON mpe.people = pe.id
            WHERE m.date BETWEEN $1 AND $2
            GROUP BY m.date, m."time", m.id, food_source_name, meal_type, m.notes
        ),
        meal_formatted AS (
            SELECT 
                date,
                meal_time,
                TRIM(CONCAT_WS(' ',
                    food_source_name,
                    CASE WHEN meal_type IS NOT NULL AND meal_type != 'cooked' THEN '(' || meal_type || ')' END,
                    CASE WHEN array_length(people_names, 1) > 0 THEN 'with ' || array_to_string(people_names, ', ') END,
                    CASE WHEN notes IS NOT NULL AND notes != '' THEN '- ' || notes END
                )) as formatted_meal
            FROM meal_aggregated
            WHERE food_source_name IS NOT NULL
        ),
        event_aggregated AS (
            SELECT 
                e.date,
                a.name as activity_name,
                e.measure,
                e.location,
                e.notes,
                array_agg(pe.name ORDER BY pe.name) FILTER (WHERE pe.name IS NOT NULL) as people_names
            FROM event e
            JOIN activity a ON e.activity = a.id
            LEFT JOIN event_people ep ON e.id = ep.event
            LEFT JOIN people pe ON ep.people = pe.id
            WHERE e.date BETWEEN $1 AND $2
            GROUP BY e.date, e.id, a.name, e.measure, e.location, e.notes
        ),
        event_formatted AS (
            SELECT 
                date,
                TRIM(CONCAT_WS(' ',
                    activity_name,
                    CASE WHEN location IS NOT NULL AND location != '' THEN '@' || location END,
                    CASE WHEN measure IS NOT NULL AND measure != '' THEN 'for ' || measure END,
                    CASE WHEN array_length(people_names, 1) > 0 THEN 'with ' || array_to_string(people_names, ', ') END,
                    CASE WHEN notes IS NOT NULL AND notes != '' THEN '- ' || notes END
                )) as formatted_event
            FROM event_aggregated
        ),
        drink_aggregated AS (
            SELECT 
                d.date,
                d.name as drink_name,
                array_agg(pe.name ORDER BY pe.name) FILTER (WHERE pe.name IS NOT NULL) as people_names
            FROM drink d
            LEFT JOIN drink_people dp ON d.id = dp.drink
            LEFT JOIN people pe ON dp.people = pe.id
            WHERE d.date BETWEEN $1 AND $2
            GROUP BY d.date, d.id, d.name
        ),
        drink_formatted AS (
            SELECT 
                date,
                TRIM(CONCAT_WS(' ',
                    drink_name,
                    CASE WHEN array_length(people_names, 1) > 0 THEN 'with ' || array_to_string(people_names, ', ') END
                )) as formatted_drink
            FROM drink_aggregated
        )
        SELECT 
            dr.date,
            CASE 
                WHEN EXTRACT(dow FROM dr.date) = 0 THEN 'Sunday'
                WHEN EXTRACT(dow FROM dr.date) = 1 THEN 'Monday'
                WHEN EXTRACT(dow FROM dr.date) = 2 THEN 'Tuesday'
                WHEN EXTRACT(dow FROM dr.date) = 3 THEN 'Wednesday'
                WHEN EXTRACT(dow FROM dr.date) = 4 THEN 'Thursday'
                WHEN EXTRACT(dow FROM dr.date) = 5 THEN 'Friday'
                WHEN EXTRACT(dow FROM dr.date) = 6 THEN 'Saturday'
            END as day_of_week,
            COALESCE(breakfast.meals, ARRAY[]::text[]) as "breakfast!",
            COALESCE(lunch.meals, ARRAY[]::text[]) as "lunch!",
            COALESCE(dinner.meals, ARRAY[]::text[]) as "dinner!",
            COALESCE(drinks.drink_list, ARRAY[]::text[]) as "drinks!",
            COALESCE(events.event_list, ARRAY[]::text[]) as "events!"
        FROM date_range dr
        LEFT JOIN (
            SELECT 
                date,
                array_agg(formatted_meal) as meals
            FROM meal_formatted 
            WHERE meal_time = 'breakfast'
            GROUP BY date
        ) breakfast ON dr.date = breakfast.date
        LEFT JOIN (
            SELECT 
                date,
                array_agg(formatted_meal) as meals
            FROM meal_formatted 
            WHERE meal_time = 'lunch'
            GROUP BY date
        ) lunch ON dr.date = lunch.date
        LEFT JOIN (
            SELECT 
                date,
                array_agg(formatted_meal) as meals
            FROM meal_formatted 
            WHERE meal_time = 'dinner'
            GROUP BY date
        ) dinner ON dr.date = dinner.date
        LEFT JOIN (
            SELECT 
                date,
                array_agg(formatted_drink) as drink_list
            FROM drink_formatted
            GROUP BY date
        ) drinks ON dr.date = drinks.date
        LEFT JOIN (
            SELECT 
                date,
                array_agg(formatted_event) as event_list
            FROM event_formatted
            GROUP BY date
        ) events ON dr.date = events.date
        ORDER BY dr.date
        "#,
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await?;

    let result = summaries.into_iter().map(|row| DailySummary {
        date: row.date.expect("Date should always be present"),
        day_of_week: row.day_of_week.unwrap_or_else(|| "Unknown".to_string()),
        breakfast: row.breakfast,
        lunch: row.lunch,
        dinner: row.dinner,
        drinks: row.drinks,
        events: row.events,
    }).collect();

    Ok(result)
}

