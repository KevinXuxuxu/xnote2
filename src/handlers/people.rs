use crate::models::people::People;
use actix_web::{web, HttpResponse, Result};
use sqlx::PgPool;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/people")
            .route(web::get().to(get_people))
            .route(web::post().to(create_person)),
    )
    .service(
        web::resource("/people/{id}")
            .route(web::get().to(get_person))
            .route(web::put().to(update_person))
            .route(web::delete().to(delete_person)),
    );
}

async fn get_people(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    match sqlx::query_as::<_, People>("SELECT id, name, notes FROM people ORDER BY name")
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(people) => Ok(HttpResponse::Ok().json(people)),
        Err(e) => {
            log::error!("Failed to fetch people: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch people"
            })))
        }
    }
}

async fn create_person(_pool: web::Data<PgPool>) -> Result<HttpResponse> {
    Ok(HttpResponse::Created().json(serde_json::json!({
        "message": "Create person - TODO: implement"
    })))
}

async fn get_person(pool: web::Data<PgPool>, path: web::Path<i32>) -> Result<HttpResponse> {
    let person_id = path.into_inner();

    match sqlx::query_as::<_, People>("SELECT id, name, notes FROM people WHERE id = $1")
        .bind(person_id)
        .fetch_optional(pool.get_ref())
        .await
    {
        Ok(Some(person)) => Ok(HttpResponse::Ok().json(person)),
        Ok(None) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Person not found"
        }))),
        Err(e) => {
            log::error!("Failed to fetch person {}: {}", person_id, e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch person"
            })))
        }
    }
}

async fn update_person(_pool: web::Data<PgPool>, _path: web::Path<i32>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Update person - TODO: implement"
    })))
}

async fn delete_person(_pool: web::Data<PgPool>, _path: web::Path<i32>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Delete person - TODO: implement"
    })))
}
