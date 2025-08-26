use crate::models::people::People;
use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;
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
    match sqlx::query_as::<_, People>("SELECT id, name, notes FROM people ORDER BY id")
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

#[derive(Debug, Deserialize)]
pub struct CreatePerson {
    pub name: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePerson {
    pub name: Option<String>,
    pub notes: Option<String>,
}

async fn create_person(
    pool: web::Data<PgPool>,
    person_data: web::Json<CreatePerson>,
) -> Result<HttpResponse> {
    match sqlx::query!(
        "INSERT INTO people (name, notes) VALUES ($1, $2) RETURNING id",
        person_data.name,
        person_data.notes
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(row) => Ok(HttpResponse::Created().json(serde_json::json!({
            "id": row.id,
            "message": "Person created successfully"
        }))),
        Err(e) => {
            log::error!("Failed to create person: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create person"
            })))
        }
    }
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

async fn update_person(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    person_data: web::Json<UpdatePerson>,
) -> Result<HttpResponse> {
    let person_id = path.into_inner();

    // Handle different update cases explicitly
    let result = match (&person_data.name, &person_data.notes) {
        (Some(name), Some(notes)) => {
            sqlx::query!(
                "UPDATE people SET name = $1, notes = $2 WHERE id = $3",
                name,
                notes,
                person_id
            )
            .execute(pool.get_ref())
            .await
        }
        (Some(name), None) => {
            sqlx::query!(
                "UPDATE people SET name = $1 WHERE id = $2",
                name,
                person_id
            )
            .execute(pool.get_ref())
            .await
        }
        (None, Some(notes)) => {
            sqlx::query!(
                "UPDATE people SET notes = $1 WHERE id = $2",
                notes,
                person_id
            )
            .execute(pool.get_ref())
            .await
        }
        (None, None) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "No fields to update"
            })));
        }
    };

    match result {
        Ok(query_result) => {
            if query_result.rows_affected() == 0 {
                Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Person not found"
                })))
            } else {
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "message": "Person updated successfully"
                })))
            }
        }
        Err(e) => {
            log::error!("Failed to update person {}: {}", person_id, e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update person"
            })))
        }
    }
}

async fn delete_person(pool: web::Data<PgPool>, path: web::Path<i32>) -> Result<HttpResponse> {
    let person_id = path.into_inner();

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Failed to start transaction: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete person"
            })));
        }
    };

    // First check if the person exists
    match sqlx::query!("SELECT id FROM people WHERE id = $1", person_id)
        .fetch_optional(&mut *tx)
        .await
    {
        Ok(Some(_)) => {
            // Person exists, continue with deletion
        }
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Person not found"
            })));
        }
        Err(e) => {
            log::error!("Failed to check if person exists: {}", e);
            if let Err(rollback_err) = tx.rollback().await {
                log::error!("Failed to rollback transaction: {}", rollback_err);
            }
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete person"
            })));
        }
    }

    // Delete person relationships (CASCADE should handle this, but being explicit)
    let relationship_tables = ["meal_people", "event_people", "drink_people"];

    for table in relationship_tables {
        if let Err(e) = sqlx::query(&format!("DELETE FROM {} WHERE people = $1", table))
            .bind(person_id)
            .execute(&mut *tx)
            .await
        {
            log::error!("Failed to delete {} relationships: {}", table, e);
            if let Err(rollback_err) = tx.rollback().await {
                log::error!("Failed to rollback transaction: {}", rollback_err);
            }
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete person"
            })));
        }
    }

    // Delete the person itself
    if let Err(e) = sqlx::query!("DELETE FROM people WHERE id = $1", person_id)
        .execute(&mut *tx)
        .await
    {
        log::error!("Failed to delete person: {}", e);
        if let Err(rollback_err) = tx.rollback().await {
            log::error!("Failed to rollback transaction: {}", rollback_err);
        }
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to delete person"
        })));
    }

    // Commit the transaction
    if let Err(e) = tx.commit().await {
        log::error!("Failed to commit transaction: {}", e);
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to delete person"
        })));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Person deleted successfully"
    })))
}
