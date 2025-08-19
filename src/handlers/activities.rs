use actix_web::{web, HttpResponse, Result};
use sqlx::PgPool;
use crate::models::activity::{Activity, CreateActivity, UpdateActivity};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/activities")
            .route(web::get().to(get_activities))
            .route(web::post().to(create_activity))
    )
    .service(
        web::resource("/activities/{id}")
            .route(web::get().to(get_activity))
            .route(web::put().to(update_activity))
            .route(web::delete().to(delete_activity))
    );
}

async fn get_activities(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    match sqlx::query_as::<_, Activity>(
        "SELECT id, name, type FROM activity ORDER BY id"
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(activities) => Ok(HttpResponse::Ok().json(activities)),
        Err(e) => {
            log::error!("Failed to fetch activities: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch activities"
            })))
        }
    }
}

async fn create_activity(
    pool: web::Data<PgPool>,
    activity_data: web::Json<CreateActivity>
) -> Result<HttpResponse> {
    match sqlx::query_as::<_, Activity>(
        r#"
        INSERT INTO activity (name, type)
        VALUES ($1, $2)
        RETURNING id, name, type
        "#
    )
    .bind(&activity_data.name)
    .bind(&activity_data.activity_type)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(activity) => Ok(HttpResponse::Created().json(activity)),
        Err(e) => {
            log::error!("Failed to create activity: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create activity"
            })))
        }
    }
}

async fn get_activity(pool: web::Data<PgPool>, path: web::Path<i32>) -> Result<HttpResponse> {
    let activity_id = path.into_inner();
    
    match sqlx::query_as::<_, Activity>(
        "SELECT id, name, type FROM activity WHERE id = $1"
    )
    .bind(activity_id)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(activity)) => Ok(HttpResponse::Ok().json(activity)),
        Ok(None) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Activity not found"
        }))),
        Err(e) => {
            log::error!("Failed to fetch activity {}: {}", activity_id, e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch activity"
            })))
        }
    }
}

async fn update_activity(
    pool: web::Data<PgPool>, 
    path: web::Path<i32>,
    activity_data: web::Json<UpdateActivity>
) -> Result<HttpResponse> {
    let activity_id = path.into_inner();
    
    // Build dynamic update query
    let mut query_parts = Vec::new();
    let mut param_index = 2; // Start from $2 since $1 is the ID
    
    if activity_data.name.is_some() {
        query_parts.push(format!("name = ${}", param_index));
        param_index += 1;
    }
    if activity_data.activity_type.is_some() {
        query_parts.push(format!("type = ${}", param_index));
    }
    
    if query_parts.is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "No fields to update"
        })));
    }
    
    let query = format!(
        "UPDATE activity SET {} WHERE id = $1 RETURNING id, name, type",
        query_parts.join(", ")
    );
    
    let mut query_builder = sqlx::query_as::<_, Activity>(&query).bind(activity_id);
    
    // Bind parameters in the same order
    if let Some(ref name) = activity_data.name {
        query_builder = query_builder.bind(name);
    }
    if let Some(ref activity_type) = activity_data.activity_type {
        query_builder = query_builder.bind(activity_type);
    }
    
    match query_builder.fetch_optional(pool.get_ref()).await {
        Ok(Some(activity)) => Ok(HttpResponse::Ok().json(activity)),
        Ok(None) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Activity not found"
        }))),
        Err(e) => {
            log::error!("Failed to update activity {}: {}", activity_id, e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update activity"
            })))
        }
    }
}

async fn delete_activity(pool: web::Data<PgPool>, path: web::Path<i32>) -> Result<HttpResponse> {
    let activity_id = path.into_inner();
    
    // First check if activity exists
    let activity_exists = sqlx::query!(
        "SELECT id FROM activity WHERE id = $1",
        activity_id
    )
    .fetch_optional(pool.get_ref())
    .await;
    
    match activity_exists {
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Activity not found"
            })));
        },
        Err(e) => {
            log::error!("Failed to check activity existence {}: {}", activity_id, e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete activity"
            })));
        },
        Ok(Some(_)) => {} // Activity exists, continue
    }
    
    // Check if activity is referenced in events
    let event_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM event WHERE activity = $1",
        activity_id
    )
    .fetch_one(pool.get_ref())
    .await;
    
    match event_count {
        Ok(result) => {
            if result.count.unwrap_or(0) > 0 {
                return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Cannot delete activity: it is referenced in {} event(s). Please delete those events first.", result.count.unwrap_or(0))
                })));
            }
        },
        Err(e) => {
            log::error!("Failed to check event references for activity {}: {}", activity_id, e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete activity"
            })));
        }
    }
    
    // Now safe to delete the activity
    match sqlx::query!(
        "DELETE FROM activity WHERE id = $1",
        activity_id
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "message": "Activity deleted successfully"
                })))
            } else {
                // This shouldn't happen since we checked existence above
                Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Activity not found"
                })))
            }
        },
        Err(e) => {
            log::error!("Failed to delete activity {}: {}", activity_id, e);
            
            // Check if it's a foreign key constraint error
            let error_message = if e.to_string().contains("foreign key") {
                "Cannot delete activity: it is still referenced by other records"
            } else {
                "Failed to delete activity"
            };
            
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": error_message
            })))
        }
    }
}