use actix_web::{web, HttpResponse, Result};
use sqlx::PgPool;
use crate::models::meal::Meal;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/meals")
            .route(web::get().to(get_meals))
            .route(web::post().to(create_meal))
    )
    .service(
        web::resource("/meals/{id}")
            .route(web::get().to(get_meal))
            .route(web::put().to(update_meal))
            .route(web::delete().to(delete_meal))
    );
}

async fn get_meals(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    match sqlx::query_as::<_, Meal>(
        "SELECT id, date, \"time\", notes FROM meal ORDER BY date DESC, \"time\""
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(meals) => Ok(HttpResponse::Ok().json(meals)),
        Err(e) => {
            log::error!("Failed to fetch meals: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch meals"
            })))
        }
    }
}

async fn create_meal(_pool: web::Data<PgPool>) -> Result<HttpResponse> {
    Ok(HttpResponse::Created().json(serde_json::json!({
        "message": "Create meal - TODO: implement"
    })))
}

async fn get_meal(pool: web::Data<PgPool>, path: web::Path<i32>) -> Result<HttpResponse> {
    let meal_id = path.into_inner();
    
    match sqlx::query_as::<_, Meal>(
        "SELECT id, date, \"time\", notes FROM meal WHERE id = $1"
    )
    .bind(meal_id)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(meal)) => Ok(HttpResponse::Ok().json(meal)),
        Ok(None) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Meal not found"
        }))),
        Err(e) => {
            log::error!("Failed to fetch meal {}: {}", meal_id, e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch meal"
            })))
        }
    }
}

async fn update_meal(_pool: web::Data<PgPool>, _path: web::Path<i32>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Update meal - TODO: implement"
    })))
}

async fn delete_meal(_pool: web::Data<PgPool>, _path: web::Path<i32>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Delete meal - TODO: implement"
    })))
}