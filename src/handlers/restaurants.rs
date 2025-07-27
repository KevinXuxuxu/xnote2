use actix_web::{web, HttpResponse, Result};
use sqlx::PgPool;
use crate::models::restaurant::Restaurant;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/restaurants")
            .route(web::get().to(get_restaurants))
            .route(web::post().to(create_restaurant))
    )
    .service(
        web::resource("/restaurants/{id}")
            .route(web::get().to(get_restaurant))
            .route(web::put().to(update_restaurant))
            .route(web::delete().to(delete_restaurant))
    );
}

async fn get_restaurants(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    match sqlx::query_as::<_, Restaurant>(
        "SELECT id, name, location, type, price FROM restaurant ORDER BY name"
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(restaurants) => Ok(HttpResponse::Ok().json(restaurants)),
        Err(e) => {
            log::error!("Failed to fetch restaurants: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch restaurants"
            })))
        }
    }
}

async fn create_restaurant(_pool: web::Data<PgPool>) -> Result<HttpResponse> {
    Ok(HttpResponse::Created().json(serde_json::json!({
        "message": "Create restaurant - TODO: implement"
    })))
}

async fn get_restaurant(pool: web::Data<PgPool>, path: web::Path<i32>) -> Result<HttpResponse> {
    let restaurant_id = path.into_inner();
    
    match sqlx::query_as::<_, Restaurant>(
        "SELECT id, name, location, type, price FROM restaurant WHERE id = $1"
    )
    .bind(restaurant_id)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(restaurant)) => Ok(HttpResponse::Ok().json(restaurant)),
        Ok(None) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Restaurant not found"
        }))),
        Err(e) => {
            log::error!("Failed to fetch restaurant {}: {}", restaurant_id, e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch restaurant"
            })))
        }
    }
}

async fn update_restaurant(_pool: web::Data<PgPool>, _path: web::Path<i32>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Update restaurant - TODO: implement"
    })))
}

async fn delete_restaurant(_pool: web::Data<PgPool>, _path: web::Path<i32>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Delete restaurant - TODO: implement"
    })))
}