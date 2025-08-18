use actix_web::{web, HttpResponse, Result};
use sqlx::PgPool;
use crate::models::restaurant::FoodType;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/food-types")
            .route(web::get().to(get_food_types))
            .route(web::post().to(create_food_type))
    )
    .service(
        web::resource("/food-types/{name}")
            .route(web::delete().to(delete_food_type))
    );
}

async fn get_food_types(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    match sqlx::query_as::<_, FoodType>(
        "SELECT name FROM food_type ORDER BY name"
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(food_types) => Ok(HttpResponse::Ok().json(food_types)),
        Err(e) => {
            log::error!("Failed to fetch food types: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch food types"
            })))
        }
    }
}

async fn create_food_type(_pool: web::Data<PgPool>) -> Result<HttpResponse> {
    Ok(HttpResponse::Created().json(serde_json::json!({
        "message": "Create food type - TODO: implement"
    })))
}

async fn delete_food_type(_pool: web::Data<PgPool>, _path: web::Path<String>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Delete food type - TODO: implement"
    })))
}