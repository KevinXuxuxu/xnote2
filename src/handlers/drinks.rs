use actix_web::{web, HttpResponse, Result};
use sqlx::PgPool;
use crate::models::drink::Drink;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/drinks")
            .route(web::get().to(get_drinks))
            .route(web::post().to(create_drink))
    )
    .service(
        web::resource("/drinks/{id}")
            .route(web::get().to(get_drink))
            .route(web::put().to(update_drink))
            .route(web::delete().to(delete_drink))
    );
}

async fn get_drinks(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    match sqlx::query_as::<_, Drink>(
        "SELECT id, name, date FROM drink ORDER BY date DESC"
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(drinks) => Ok(HttpResponse::Ok().json(drinks)),
        Err(e) => {
            log::error!("Failed to fetch drinks: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch drinks"
            })))
        }
    }
}

async fn create_drink(_pool: web::Data<PgPool>) -> Result<HttpResponse> {
    Ok(HttpResponse::Created().json(serde_json::json!({
        "message": "Create drink - TODO: implement"
    })))
}

async fn get_drink(pool: web::Data<PgPool>, path: web::Path<i32>) -> Result<HttpResponse> {
    let drink_id = path.into_inner();
    
    match sqlx::query_as::<_, Drink>(
        "SELECT id, name, date FROM drink WHERE id = $1"
    )
    .bind(drink_id)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(drink)) => Ok(HttpResponse::Ok().json(drink)),
        Ok(None) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Drink not found"
        }))),
        Err(e) => {
            log::error!("Failed to fetch drink {}: {}", drink_id, e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch drink"
            })))
        }
    }
}

async fn update_drink(_pool: web::Data<PgPool>, _path: web::Path<i32>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Update drink - TODO: implement"
    })))
}

async fn delete_drink(_pool: web::Data<PgPool>, _path: web::Path<i32>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Delete drink - TODO: implement"
    })))
}