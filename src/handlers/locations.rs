use crate::models::location::Location;
use actix_web::{HttpResponse, Result, web};
use sqlx::PgPool;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/locations")
            .route(web::get().to(get_locations))
            .route(web::post().to(create_location)),
    )
    .service(web::resource("/locations/{name}").route(web::delete().to(delete_location)));
}

async fn get_locations(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    match sqlx::query_as::<_, Location>("SELECT name FROM location ORDER BY name")
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(locations) => Ok(HttpResponse::Ok().json(locations)),
        Err(e) => {
            log::error!("Failed to fetch locations: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch locations"
            })))
        }
    }
}

async fn create_location(_pool: web::Data<PgPool>) -> Result<HttpResponse> {
    Ok(HttpResponse::Created().json(serde_json::json!({
        "message": "Create location - TODO: implement"
    })))
}

async fn delete_location(
    _pool: web::Data<PgPool>,
    _path: web::Path<String>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Delete location - TODO: implement"
    })))
}
