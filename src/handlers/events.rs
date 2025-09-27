use crate::models::detail::{ActivityDetail, EventDetail};
use crate::models::event::{CreateEvent, CreateEventResponse, Event};
use crate::models::people::People;
use actix_web::{web, HttpResponse, Result};
use sqlx::PgPool;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/events")
            .route(web::get().to(get_events))
            .route(web::post().to(create_event)),
    )
    .service(
        web::resource("/events/{id}")
            .route(web::get().to(get_event))
            .route(web::put().to(update_event))
            .route(web::delete().to(delete_event)),
    )
    .service(web::resource("/events/{id}/details").route(web::get().to(get_event_details)));
}

async fn get_events(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    match sqlx::query_as::<_, Event>(
        "SELECT id, date, activity, measure, location, notes FROM event ORDER BY date DESC",
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(events) => Ok(HttpResponse::Ok().json(events)),
        Err(e) => {
            log::error!("Failed to fetch events: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch events"
            })))
        }
    }
}

async fn create_event(
    pool: web::Data<PgPool>,
    event_data: web::Json<CreateEvent>,
) -> Result<HttpResponse> {
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Failed to start transaction: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create event"
            })));
        }
    };

    // Insert the event record
    let event_result = sqlx::query!(
        r#"
        INSERT INTO event (date, activity, measure, location, notes)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
        event_data.date,
        event_data.activity_id,
        event_data.measure,
        event_data.location,
        event_data.notes
    )
    .fetch_one(&mut *tx)
    .await;

    let event_id = match event_result {
        Ok(row) => row.id,
        Err(e) => {
            log::error!("Failed to insert event: {}", e);
            if let Err(rollback_err) = tx.rollback().await {
                log::error!("Failed to rollback transaction: {}", rollback_err);
            }
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create event"
            })));
        }
    };

    // Insert event-people relationships
    for person_id in &event_data.people_ids {
        if let Err(e) = sqlx::query!(
            "INSERT INTO event_people (event, people) VALUES ($1, $2)",
            event_id,
            person_id
        )
        .execute(&mut *tx)
        .await
        {
            log::error!("Failed to insert event_people relationship: {}", e);
            if let Err(rollback_err) = tx.rollback().await {
                log::error!("Failed to rollback transaction: {}", rollback_err);
            }
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create event"
            })));
        }
    }

    // Commit the transaction
    if let Err(e) = tx.commit().await {
        log::error!("Failed to commit transaction: {}", e);
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to create event"
        })));
    }

    Ok(HttpResponse::Created().json(CreateEventResponse {
        id: event_id,
        message: "Event created successfully".to_string(),
    }))
}

async fn get_event(pool: web::Data<PgPool>, path: web::Path<i32>) -> Result<HttpResponse> {
    let event_id = path.into_inner();

    match sqlx::query_as::<_, Event>(
        "SELECT id, date, activity, measure, location, notes FROM event WHERE id = $1",
    )
    .bind(event_id)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(event)) => Ok(HttpResponse::Ok().json(event)),
        Ok(None) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Event not found"
        }))),
        Err(e) => {
            log::error!("Failed to fetch event {}: {}", event_id, e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch event"
            })))
        }
    }
}

async fn update_event(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    event_data: web::Json<CreateEvent>,
) -> Result<HttpResponse> {
    let event_id = path.into_inner();
    
    // Start transaction
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Failed to start transaction: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update event"
            })));
        }
    };

    // Step 1: Check if event exists
    let existing_event = match sqlx::query_as::<_, Event>(
        "SELECT id, date, activity, measure, location, notes FROM event WHERE id = $1"
    )
    .bind(event_id)
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(Some(event)) => event,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Event not found"
            })));
        }
        Err(e) => {
            log::error!("Failed to fetch event: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update event"
            })));
        }
    };

    // Step 2: Update the main event record
    if let Err(e) = sqlx::query!(
        r#"
        UPDATE event 
        SET date = $1, activity = $2, measure = $3, location = $4, notes = $5 
        WHERE id = $6
        "#,
        event_data.date,
        event_data.activity_id,
        event_data.measure,
        event_data.location,
        event_data.notes,
        event_id
    )
    .execute(&mut *tx)
    .await
    {
        log::error!("Failed to update event: {}", e);
        let _ = tx.rollback().await;
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to update event"
        })));
    }

    // Step 3: Remove existing people relationships
    let _ = sqlx::query!("DELETE FROM event_people WHERE event = $1", event_id)
        .execute(&mut *tx)
        .await;

    // Step 4: Insert new people relationships
    for person_id in &event_data.people_ids {
        if let Err(e) = sqlx::query!(
            "INSERT INTO event_people (event, people) VALUES ($1, $2)",
            event_id,
            person_id
        )
        .execute(&mut *tx)
        .await
        {
            log::error!("Failed to update event people relationship: {}", e);
            let _ = tx.rollback().await;
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update event people relationships"
            })));
        }
    }

    // Step 5: Commit transaction
    if let Err(e) = tx.commit().await {
        log::error!("Failed to commit transaction: {}", e);
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to update event"
        })));
    }

    // Return success response
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Event updated successfully",
        "id": event_id
    })))
}

async fn delete_event(pool: web::Data<PgPool>, path: web::Path<i32>) -> Result<HttpResponse> {
    let event_id = path.into_inner();

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Failed to start transaction: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete event"
            })));
        }
    };

    // First check if the event exists
    match sqlx::query!("SELECT id FROM event WHERE id = $1", event_id)
        .fetch_optional(&mut *tx)
        .await
    {
        Ok(Some(_)) => {
            // Event exists, continue with deletion
        }
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Event not found"
            })));
        }
        Err(e) => {
            log::error!("Failed to check if event exists: {}", e);
            if let Err(rollback_err) = tx.rollback().await {
                log::error!("Failed to rollback transaction: {}", rollback_err);
            }
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete event"
            })));
        }
    }

    // Delete event_people relationships (will cascade automatically, but explicit is better)
    if let Err(e) = sqlx::query!("DELETE FROM event_people WHERE event = $1", event_id)
        .execute(&mut *tx)
        .await
    {
        log::error!("Failed to delete event_people relationships: {}", e);
        if let Err(rollback_err) = tx.rollback().await {
            log::error!("Failed to rollback transaction: {}", rollback_err);
        }
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to delete event"
        })));
    }

    // Delete the event itself
    if let Err(e) = sqlx::query!("DELETE FROM event WHERE id = $1", event_id)
        .execute(&mut *tx)
        .await
    {
        log::error!("Failed to delete event: {}", e);
        if let Err(rollback_err) = tx.rollback().await {
            log::error!("Failed to rollback transaction: {}", rollback_err);
        }
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to delete event"
        })));
    }

    // Commit the transaction
    if let Err(e) = tx.commit().await {
        log::error!("Failed to commit transaction: {}", e);
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to delete event"
        })));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Event deleted successfully"
    })))
}

async fn get_event_details(pool: web::Data<PgPool>, path: web::Path<i32>) -> Result<HttpResponse> {
    let event_id = path.into_inner();

    // Get event with activity details in a single query
    let event_query = sqlx::query!(
        r#"
        SELECT 
            e.id, e.date, e.measure, e.location, e.notes,
            a.id as activity_id, a.name as activity_name, a.type as activity_type
        FROM event e
        JOIN activity a ON e.activity = a.id
        WHERE e.id = $1
        "#,
        event_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    let event_row = match event_query {
        Ok(Some(event)) => event,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Event not found"
            })));
        }
        Err(e) => {
            log::error!("Failed to fetch event {}: {}", event_id, e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch event"
            })));
        }
    };

    // Get people associated with this event
    let people_result = sqlx::query_as::<_, People>(
        r#"
        SELECT p.id, p.name, p.notes
        FROM people p
        JOIN event_people ep ON p.id = ep.people
        WHERE ep.event = $1
        ORDER BY p.name
        "#,
    )
    .bind(event_id)
    .fetch_all(pool.get_ref())
    .await;

    let people = match people_result {
        Ok(people) => people,
        Err(e) => {
            log::error!("Failed to fetch event people {}: {}", event_id, e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch event details"
            })));
        }
    };

    let event_detail = EventDetail {
        id: event_row.id,
        date: event_row.date,
        activity: ActivityDetail {
            id: event_row.activity_id,
            name: event_row.activity_name,
            activity_type: event_row.activity_type,
        },
        measure: event_row.measure,
        location: event_row.location,
        notes: event_row.notes,
        people,
    };

    Ok(HttpResponse::Ok().json(event_detail))
}
