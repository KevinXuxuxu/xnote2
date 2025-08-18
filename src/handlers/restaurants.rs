use actix_web::{web, HttpResponse, Result};
use sqlx::PgPool;
use crate::models::restaurant::{Restaurant, CreateRestaurant, UpdateRestaurant};

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

async fn create_restaurant(
    pool: web::Data<PgPool>,
    restaurant_data: web::Json<CreateRestaurant>
) -> Result<HttpResponse> {
    match sqlx::query_as::<_, Restaurant>(
        r#"
        INSERT INTO restaurant (name, location, type, price)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, location, type, price
        "#
    )
    .bind(&restaurant_data.name)
    .bind(&restaurant_data.location)
    .bind(&restaurant_data.food_type)
    .bind(restaurant_data.price)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(restaurant) => Ok(HttpResponse::Created().json(restaurant)),
        Err(e) => {
            log::error!("Failed to create restaurant: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create restaurant"
            })))
        }
    }
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

async fn update_restaurant(
    pool: web::Data<PgPool>, 
    path: web::Path<i32>,
    restaurant_data: web::Json<UpdateRestaurant>
) -> Result<HttpResponse> {
    let restaurant_id = path.into_inner();
    
    // Build dynamic update query
    let mut query_parts = Vec::new();
    let mut param_index = 2; // Start from $2 since $1 is the ID
    
    if restaurant_data.name.is_some() {
        query_parts.push(format!("name = ${}", param_index));
        param_index += 1;
    }
    if restaurant_data.location.is_some() {
        query_parts.push(format!("location = ${}", param_index));
        param_index += 1;
    }
    if restaurant_data.food_type.is_some() {
        query_parts.push(format!("type = ${}", param_index));
        param_index += 1;
    }
    if restaurant_data.price.is_some() {
        query_parts.push(format!("price = ${}", param_index));
        param_index += 1;
    }
    
    if query_parts.is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "No fields to update"
        })));
    }
    
    let query = format!(
        "UPDATE restaurant SET {} WHERE id = $1 RETURNING id, name, location, type, price",
        query_parts.join(", ")
    );
    
    let mut query_builder = sqlx::query_as::<_, Restaurant>(&query).bind(restaurant_id);
    
    // Bind parameters in the same order
    if let Some(ref name) = restaurant_data.name {
        query_builder = query_builder.bind(name);
    }
    if let Some(ref location) = restaurant_data.location {
        query_builder = query_builder.bind(location);
    }
    if let Some(ref food_type) = restaurant_data.food_type {
        query_builder = query_builder.bind(food_type);
    }
    if restaurant_data.price.is_some() {
        query_builder = query_builder.bind(restaurant_data.price);
    }
    
    match query_builder.fetch_optional(pool.get_ref()).await {
        Ok(Some(restaurant)) => Ok(HttpResponse::Ok().json(restaurant)),
        Ok(None) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Restaurant not found"
        }))),
        Err(e) => {
            log::error!("Failed to update restaurant {}: {}", restaurant_id, e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update restaurant"
            })))
        }
    }
}

async fn delete_restaurant(pool: web::Data<PgPool>, path: web::Path<i32>) -> Result<HttpResponse> {
    let restaurant_id = path.into_inner();
    
    // First check if restaurant exists
    let restaurant_exists = sqlx::query!(
        "SELECT id FROM restaurant WHERE id = $1",
        restaurant_id
    )
    .fetch_optional(pool.get_ref())
    .await;
    
    match restaurant_exists {
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Restaurant not found"
            })));
        },
        Err(e) => {
            log::error!("Failed to check restaurant existence {}: {}", restaurant_id, e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete restaurant"
            })));
        },
        Ok(Some(_)) => {} // Restaurant exists, continue
    }
    
    // Check if restaurant is referenced in meals
    let meal_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM meal_restaurant WHERE restaurant = $1",
        restaurant_id
    )
    .fetch_one(pool.get_ref())
    .await;
    
    match meal_count {
        Ok(result) => {
            if result.count.unwrap_or(0) > 0 {
                return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Cannot delete restaurant: it is referenced in {} meal(s). Please delete those meals first.", result.count.unwrap_or(0))
                })));
            }
        },
        Err(e) => {
            log::error!("Failed to check meal references for restaurant {}: {}", restaurant_id, e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete restaurant"
            })));
        }
    }
    
    // Now safe to delete the restaurant
    match sqlx::query!(
        "DELETE FROM restaurant WHERE id = $1",
        restaurant_id
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "message": "Restaurant deleted successfully"
                })))
            } else {
                // This shouldn't happen since we checked existence above
                Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Restaurant not found"
                })))
            }
        },
        Err(e) => {
            log::error!("Failed to delete restaurant {}: {}", restaurant_id, e);
            
            // Check if it's a foreign key constraint error
            let error_message = if e.to_string().contains("foreign key") {
                "Cannot delete restaurant: it is still referenced by other records"
            } else {
                "Failed to delete restaurant"
            };
            
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": error_message
            })))
        }
    }
}