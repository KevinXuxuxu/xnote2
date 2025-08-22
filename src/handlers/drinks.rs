use crate::models::detail::DrinkDetail;
use crate::models::drink::{CreateDrink, CreateDrinkResponse, Drink};
use crate::models::people::People;
use actix_web::{HttpResponse, Result, web};
use sqlx::PgPool;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/drinks")
            .route(web::get().to(get_drinks))
            .route(web::post().to(create_drink)),
    )
    .service(
        web::resource("/drinks/{id}")
            .route(web::get().to(get_drink))
            .route(web::put().to(update_drink))
            .route(web::delete().to(delete_drink)),
    )
    .service(web::resource("/drinks/{id}/details").route(web::get().to(get_drink_details)));
}

async fn get_drinks(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    match sqlx::query_as::<_, Drink>("SELECT id, name, date FROM drink ORDER BY date DESC")
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

async fn create_drink(
    pool: web::Data<PgPool>,
    drink_data: web::Json<CreateDrink>,
) -> Result<HttpResponse> {
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Failed to start transaction: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create drink"
            })));
        }
    };

    // Insert the drink record
    let drink_result = sqlx::query!(
        r#"
        INSERT INTO drink (date, name)
        VALUES ($1, $2)
        RETURNING id
        "#,
        drink_data.date,
        drink_data.name
    )
    .fetch_one(&mut *tx)
    .await;

    let drink_id = match drink_result {
        Ok(row) => row.id,
        Err(e) => {
            log::error!("Failed to insert drink: {}", e);
            if let Err(rollback_err) = tx.rollback().await {
                log::error!("Failed to rollback transaction: {}", rollback_err);
            }
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create drink"
            })));
        }
    };

    // Insert drink-people relationships
    for person_id in &drink_data.people_ids {
        if let Err(e) = sqlx::query!(
            "INSERT INTO drink_people (drink, people) VALUES ($1, $2)",
            drink_id,
            person_id
        )
        .execute(&mut *tx)
        .await
        {
            log::error!("Failed to insert drink_people relationship: {}", e);
            if let Err(rollback_err) = tx.rollback().await {
                log::error!("Failed to rollback transaction: {}", rollback_err);
            }
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create drink"
            })));
        }
    }

    // Commit the transaction
    if let Err(e) = tx.commit().await {
        log::error!("Failed to commit transaction: {}", e);
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to create drink"
        })));
    }

    Ok(HttpResponse::Created().json(CreateDrinkResponse {
        id: drink_id,
        message: "Drink created successfully".to_string(),
    }))
}

async fn get_drink(pool: web::Data<PgPool>, path: web::Path<i32>) -> Result<HttpResponse> {
    let drink_id = path.into_inner();

    match sqlx::query_as::<_, Drink>("SELECT id, name, date FROM drink WHERE id = $1")
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

async fn get_drink_details(pool: web::Data<PgPool>, path: web::Path<i32>) -> Result<HttpResponse> {
    let drink_id = path.into_inner();

    // Get drink basic info
    let drink_query = sqlx::query!("SELECT id, name, date FROM drink WHERE id = $1", drink_id)
        .fetch_optional(pool.get_ref())
        .await;

    let drink = match drink_query {
        Ok(Some(drink)) => drink,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Drink not found"
            })));
        }
        Err(e) => {
            log::error!("Failed to fetch drink {}: {}", drink_id, e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch drink"
            })));
        }
    };

    // Get people associated with this drink
    let people_result = sqlx::query_as::<_, People>(
        r#"
        SELECT p.id, p.name, p.notes
        FROM people p
        JOIN drink_people dp ON p.id = dp.people
        WHERE dp.drink = $1
        ORDER BY p.name
        "#,
    )
    .bind(drink_id)
    .fetch_all(pool.get_ref())
    .await;

    let people = match people_result {
        Ok(people) => people,
        Err(e) => {
            log::error!("Failed to fetch drink people {}: {}", drink_id, e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch drink details"
            })));
        }
    };

    let drink_detail = DrinkDetail {
        id: drink.id,
        name: drink.name,
        date: drink.date,
        people,
    };

    Ok(HttpResponse::Ok().json(drink_detail))
}
