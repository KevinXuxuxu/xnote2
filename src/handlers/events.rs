use actix_web::{web, HttpResponse, Result};
use sqlx::PgPool;
use crate::models::event::Event;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/events")
            .route(web::get().to(get_events))
            .route(web::post().to(create_event))
    )
    .service(
        web::resource("/events/{id}")
            .route(web::get().to(get_event))
            .route(web::put().to(update_event))
            .route(web::delete().to(delete_event))
    );
}

async fn get_events(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    match sqlx::query_as::<_, Event>(
        "SELECT id, date, activity, measure, location, notes FROM event ORDER BY date DESC"
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

async fn create_event(_pool: web::Data<PgPool>) -> Result<HttpResponse> {
    Ok(HttpResponse::Created().json(serde_json::json!({
        "message": "Create event - TODO: implement"
    })))
}

async fn get_event(pool: web::Data<PgPool>, path: web::Path<i32>) -> Result<HttpResponse> {
    let event_id = path.into_inner();
    
    match sqlx::query_as::<_, Event>(
        "SELECT id, date, activity, measure, location, notes FROM event WHERE id = $1"
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

async fn update_event(_pool: web::Data<PgPool>, _path: web::Path<i32>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Update event - TODO: implement"
    })))
}

async fn delete_event(_pool: web::Data<PgPool>, _path: web::Path<i32>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Delete event - TODO: implement"
    })))
}