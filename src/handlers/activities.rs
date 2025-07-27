use actix_web::{web, HttpResponse, Result};
use sqlx::PgPool;
use crate::models::activity::Activity;

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
        "SELECT id, name, type FROM activity ORDER BY name"
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

async fn create_activity(_pool: web::Data<PgPool>) -> Result<HttpResponse> {
    Ok(HttpResponse::Created().json(serde_json::json!({
        "message": "Create activity - TODO: implement"
    })))
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

async fn update_activity(_pool: web::Data<PgPool>, _path: web::Path<i32>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Update activity - TODO: implement"
    })))
}

async fn delete_activity(_pool: web::Data<PgPool>, _path: web::Path<i32>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Delete activity - TODO: implement"
    })))
}