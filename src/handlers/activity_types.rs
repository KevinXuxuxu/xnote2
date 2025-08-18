use actix_web::{web, HttpResponse, Result};
use sqlx::PgPool;
use crate::models::activity::ActivityType;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/activity-types")
            .route(web::get().to(get_activity_types))
            .route(web::post().to(create_activity_type))
    )
    .service(
        web::resource("/activity-types/{name}")
            .route(web::delete().to(delete_activity_type))
    );
}

async fn get_activity_types(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    match sqlx::query_as::<_, ActivityType>(
        "SELECT name FROM activity_type ORDER BY name"
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(activity_types) => Ok(HttpResponse::Ok().json(activity_types)),
        Err(e) => {
            log::error!("Failed to fetch activity types: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch activity types"
            })))
        }
    }
}

async fn create_activity_type(
    pool: web::Data<PgPool>,
    activity_type_data: web::Json<ActivityType>
) -> Result<HttpResponse> {
    match sqlx::query_as::<_, ActivityType>(
        "INSERT INTO activity_type (name) VALUES ($1) RETURNING name"
    )
    .bind(&activity_type_data.name)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(activity_type) => Ok(HttpResponse::Created().json(activity_type)),
        Err(e) => {
            log::error!("Failed to create activity type: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create activity type"
            })))
        }
    }
}

async fn delete_activity_type(pool: web::Data<PgPool>, path: web::Path<String>) -> Result<HttpResponse> {
    let activity_type_name = path.into_inner();
    
    // Check if activity type is referenced in activities
    let activity_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM activity WHERE type = $1",
        activity_type_name
    )
    .fetch_one(pool.get_ref())
    .await;
    
    match activity_count {
        Ok(result) => {
            if result.count.unwrap_or(0) > 0 {
                return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Cannot delete activity type: it is referenced in {} activity(ies). Please delete those activities first.", result.count.unwrap_or(0))
                })));
            }
        },
        Err(e) => {
            log::error!("Failed to check activity references for activity type {}: {}", activity_type_name, e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete activity type"
            })));
        }
    }
    
    match sqlx::query!(
        "DELETE FROM activity_type WHERE name = $1",
        activity_type_name
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "message": "Activity type deleted successfully"
                })))
            } else {
                Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Activity type not found"
                })))
            }
        },
        Err(e) => {
            log::error!("Failed to delete activity type {}: {}", activity_type_name, e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete activity type"
            })))
        }
    }
}