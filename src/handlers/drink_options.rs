use actix_web::{web, HttpResponse, Result};
use sqlx::PgPool;
use crate::models::drink::DrinkOption;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/drink-options")
            .route(web::get().to(get_drink_options))
            .route(web::post().to(create_drink_option))
    )
    .service(
        web::resource("/drink-options/{name}")
            .route(web::delete().to(delete_drink_option))
    );
}

async fn get_drink_options(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    match sqlx::query_as::<_, DrinkOption>(
        "SELECT name FROM drink_option ORDER BY name"
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(drink_options) => Ok(HttpResponse::Ok().json(drink_options)),
        Err(e) => {
            log::error!("Failed to fetch drink options: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch drink options"
            })))
        }
    }
}

async fn create_drink_option(pool: web::Data<PgPool>, drink_option: web::Json<DrinkOption>) -> Result<HttpResponse> {
    match sqlx::query!(
        "INSERT INTO drink_option (name) VALUES ($1)",
        drink_option.name
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => Ok(HttpResponse::Created().json(&*drink_option)),
        Err(e) => {
            log::error!("Failed to create drink option: {}", e);
            if e.to_string().contains("duplicate key") {
                Ok(HttpResponse::Conflict().json(serde_json::json!({
                    "error": "Drink option already exists"
                })))
            } else {
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to create drink option"
                })))
            }
        }
    }
}

async fn delete_drink_option(pool: web::Data<PgPool>, path: web::Path<String>) -> Result<HttpResponse> {
    let drink_option_name = path.into_inner();
    
    match sqlx::query!(
        "DELETE FROM drink_option WHERE name = $1",
        drink_option_name
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "message": "Drink option deleted successfully"
                })))
            } else {
                Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Drink option not found"
                })))
            }
        }
        Err(e) => {
            log::error!("Failed to delete drink option {}: {}", drink_option_name, e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete drink option"
            })))
        }
    }
}