use crate::models::detail::{MealDetail, MealFoodSource};
use crate::models::meal::{CreateMeal, CreateMealFoodSource, CreateMealResponse, Meal};
use crate::models::{people::People, product::Product, recipe::Recipe, restaurant::Restaurant};
use actix_web::{web, HttpResponse, Result};
use serde::Deserialize;
use sqlx::PgPool;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/meals")
            .route(web::get().to(get_meals))
            .route(web::post().to(create_meal)),
    )
    .service(
        web::resource("/meals/{id}")
            .route(web::get().to(get_meal))
            .route(web::put().to(update_meal))
            .route(web::delete().to(delete_meal)),
    )
    .service(web::resource("/meals/{id}/details").route(web::get().to(get_meal_details)))
    .service(web::resource("/meals/batch/delete").route(web::post().to(delete_meals_batch)));
}

async fn get_meals(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    match sqlx::query_as::<_, Meal>(
        "SELECT id, date, \"time\", notes FROM meal ORDER BY date DESC, \"time\"",
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

async fn create_meal(
    pool: web::Data<PgPool>,
    meal_data: web::Json<CreateMeal>,
) -> Result<HttpResponse> {
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Failed to start transaction: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create meal"
            })));
        }
    };

    // Step 1: Insert the main meal record
    let meal_result = sqlx::query!(
        r#"INSERT INTO meal (date, "time", notes) VALUES ($1, $2, $3) RETURNING id"#,
        meal_data.date,
        meal_data.time,
        meal_data.notes
    )
    .fetch_one(&mut *tx)
    .await;

    let meal_id = match meal_result {
        Ok(row) => row.id,
        Err(e) => {
            log::error!("Failed to insert meal: {}", e);
            let _ = tx.rollback().await;
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create meal"
            })));
        }
    };

    // Step 2: Insert food source relationship based on type
    let food_source_result = match &meal_data.food_source {
        CreateMealFoodSource::Recipe {
            recipe_id,
            meal_type,
        } => {
            sqlx::query!(
                "INSERT INTO meal_recipe (meal, recipe, type) VALUES ($1, $2, $3)",
                meal_id,
                recipe_id,
                meal_type
            )
            .execute(&mut *tx)
            .await
        }
        CreateMealFoodSource::Product {
            product_id,
            meal_type,
        } => {
            sqlx::query!(
                "INSERT INTO meal_product (meal, product, type) VALUES ($1, $2, $3)",
                meal_id,
                product_id,
                meal_type
            )
            .execute(&mut *tx)
            .await
        }
        CreateMealFoodSource::Restaurant {
            restaurant_id,
            meal_type,
        } => {
            sqlx::query!(
                "INSERT INTO meal_restaurant (meal, restaurant, type) VALUES ($1, $2, $3)",
                meal_id,
                restaurant_id,
                meal_type
            )
            .execute(&mut *tx)
            .await
        }
    };

    if let Err(e) = food_source_result {
        log::error!("Failed to insert meal food source: {}", e);
        let _ = tx.rollback().await;
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to create meal food source"
        })));
    }

    // Step 3: Insert people relationships
    for person_id in &meal_data.people_ids {
        if let Err(e) = sqlx::query!(
            "INSERT INTO meal_people (meal, people) VALUES ($1, $2)",
            meal_id,
            person_id
        )
        .execute(&mut *tx)
        .await
        {
            log::error!("Failed to insert meal people relationship: {}", e);
            let _ = tx.rollback().await;
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create meal people relationships"
            })));
        }
    }

    // Step 4: Commit transaction
    if let Err(e) = tx.commit().await {
        log::error!("Failed to commit transaction: {}", e);
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to create meal"
        })));
    }

    // Return the created meal response
    let response = CreateMealResponse {
        id: meal_id,
        date: meal_data.date,
        time: meal_data.time.clone(),
        notes: meal_data.notes.clone(),
    };

    Ok(HttpResponse::Created().json(response))
}

async fn get_meal(pool: web::Data<PgPool>, path: web::Path<i32>) -> Result<HttpResponse> {
    let meal_id = path.into_inner();

    match sqlx::query_as::<_, Meal>("SELECT id, date, \"time\", notes FROM meal WHERE id = $1")
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

async fn update_meal(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    meal_data: web::Json<CreateMeal>,
) -> Result<HttpResponse> {
    let meal_id = path.into_inner();
    
    // Start transaction
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Failed to start transaction: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update meal"
            })));
        }
    };

    // Step 1: Check if meal exists
    let _existing_meal = match sqlx::query_as::<_, Meal>(
        "SELECT id, date, \"time\", notes FROM meal WHERE id = $1"
    )
    .bind(meal_id)
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(Some(meal)) => meal,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Meal not found"
            })));
        }
        Err(e) => {
            log::error!("Failed to fetch meal: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update meal"
            })));
        }
    };

    // Step 2: Update the main meal record
    if let Err(e) = sqlx::query!(
        r#"UPDATE meal SET date = $1, "time" = $2, notes = $3 WHERE id = $4"#,
        meal_data.date,
        meal_data.time,
        meal_data.notes,
        meal_id
    )
    .execute(&mut *tx)
    .await
    {
        log::error!("Failed to update meal: {}", e);
        let _ = tx.rollback().await;
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to update meal"
        })));
    }

    // Step 3: Remove existing food source relationships
    // Delete from all food source tables (only one will have data)
    let _ = sqlx::query!("DELETE FROM meal_recipe WHERE meal = $1", meal_id)
        .execute(&mut *tx)
        .await;
    let _ = sqlx::query!("DELETE FROM meal_product WHERE meal = $1", meal_id)
        .execute(&mut *tx)
        .await;
    let _ = sqlx::query!("DELETE FROM meal_restaurant WHERE meal = $1", meal_id)
        .execute(&mut *tx)
        .await;

    // Step 4: Insert new food source relationship
    let food_source_result = match &meal_data.food_source {
        CreateMealFoodSource::Recipe {
            recipe_id,
            meal_type,
        } => {
            sqlx::query!(
                "INSERT INTO meal_recipe (meal, recipe, type) VALUES ($1, $2, $3)",
                meal_id,
                recipe_id,
                meal_type
            )
            .execute(&mut *tx)
            .await
        }
        CreateMealFoodSource::Product {
            product_id,
            meal_type,
        } => {
            sqlx::query!(
                "INSERT INTO meal_product (meal, product, type) VALUES ($1, $2, $3)",
                meal_id,
                product_id,
                meal_type
            )
            .execute(&mut *tx)
            .await
        }
        CreateMealFoodSource::Restaurant {
            restaurant_id,
            meal_type,
        } => {
            sqlx::query!(
                "INSERT INTO meal_restaurant (meal, restaurant, type) VALUES ($1, $2, $3)",
                meal_id,
                restaurant_id,
                meal_type
            )
            .execute(&mut *tx)
            .await
        }
    };

    if let Err(e) = food_source_result {
        log::error!("Failed to update meal food source: {}", e);
        let _ = tx.rollback().await;
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to update meal food source"
        })));
    }

    // Step 5: Remove existing people relationships
    let _ = sqlx::query!("DELETE FROM meal_people WHERE meal = $1", meal_id)
        .execute(&mut *tx)
        .await;

    // Step 6: Insert new people relationships
    for person_id in &meal_data.people_ids {
        if let Err(e) = sqlx::query!(
            "INSERT INTO meal_people (meal, people) VALUES ($1, $2)",
            meal_id,
            person_id
        )
        .execute(&mut *tx)
        .await
        {
            log::error!("Failed to update meal people relationship: {}", e);
            let _ = tx.rollback().await;
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update meal people relationships"
            })));
        }
    }

    // Step 7: Commit transaction
    if let Err(e) = tx.commit().await {
        log::error!("Failed to commit transaction: {}", e);
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to update meal"
        })));
    }

    // Return success response
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Meal updated successfully",
        "id": meal_id
    })))
}

async fn delete_meal(pool: web::Data<PgPool>, path: web::Path<i32>) -> Result<HttpResponse> {
    let meal_id = path.into_inner();

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Failed to start transaction: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete meal"
            })));
        }
    };

    // First check if the meal exists
    match sqlx::query!("SELECT id FROM meal WHERE id = $1", meal_id)
        .fetch_optional(&mut *tx)
        .await
    {
        Ok(Some(_)) => {
            // Meal exists, continue with deletion
        }
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Meal not found"
            })));
        }
        Err(e) => {
            log::error!("Failed to check if meal exists: {}", e);
            if let Err(rollback_err) = tx.rollback().await {
                log::error!("Failed to rollback transaction: {}", rollback_err);
            }
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete meal"
            })));
        }
    }

    // Delete meal relationships (CASCADE should handle this, but being explicit)
    let relationship_tables = [
        "meal_people",
        "meal_recipe",
        "meal_product",
        "meal_restaurant",
    ];

    for table in relationship_tables {
        if let Err(e) = sqlx::query(&format!("DELETE FROM {} WHERE meal = $1", table))
            .bind(meal_id)
            .execute(&mut *tx)
            .await
        {
            log::error!("Failed to delete {} relationships: {}", table, e);
            if let Err(rollback_err) = tx.rollback().await {
                log::error!("Failed to rollback transaction: {}", rollback_err);
            }
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete meal"
            })));
        }
    }

    // Delete the meal itself
    if let Err(e) = sqlx::query!("DELETE FROM meal WHERE id = $1", meal_id)
        .execute(&mut *tx)
        .await
    {
        log::error!("Failed to delete meal: {}", e);
        if let Err(rollback_err) = tx.rollback().await {
            log::error!("Failed to rollback transaction: {}", rollback_err);
        }
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to delete meal"
        })));
    }

    // Commit the transaction
    if let Err(e) = tx.commit().await {
        log::error!("Failed to commit transaction: {}", e);
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to delete meal"
        })));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Meal deleted successfully"
    })))
}

async fn get_meal_details(pool: web::Data<PgPool>, path: web::Path<i32>) -> Result<HttpResponse> {
    let meal_id = path.into_inner();

    // First get the meal basic info
    let meal_query = sqlx::query!(
        r#"SELECT id, date, "time", notes FROM meal WHERE id = $1"#,
        meal_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    let meal = match meal_query {
        Ok(Some(meal)) => meal,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Meal not found"
            })));
        }
        Err(e) => {
            log::error!("Failed to fetch meal {}: {}", meal_id, e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch meal"
            })));
        }
    };

    // Get food source (recipe, product, or restaurant)
    let food_source_result = sqlx::query!(
        r#"
        SELECT 
            CASE 
                WHEN mr.meal IS NOT NULL THEN 'recipe'
                WHEN mp.meal IS NOT NULL THEN 'product' 
                WHEN mrt.meal IS NOT NULL THEN 'restaurant'
            END as food_source_type,
            COALESCE(mr.type, mp.type, mrt.type) as meal_type,
            r.id as "recipe_id?", r.name as "recipe_name?", r.ingredients as "ingredients?", r.procedure as "procedure?", r.cautions as "cautions?",
            p.id as "product_id?", p.name as "product_name?",
            rt.id as "restaurant_id?", rt.name as "restaurant_name?", rt.location as "location?", rt.type as "restaurant_type?", rt.price as "price?"
        FROM meal m
        LEFT JOIN meal_recipe mr ON m.id = mr.meal
        LEFT JOIN recipe r ON mr.recipe = r.id
        LEFT JOIN meal_product mp ON m.id = mp.meal
        LEFT JOIN product p ON mp.product = p.id  
        LEFT JOIN meal_restaurant mrt ON m.id = mrt.meal
        LEFT JOIN restaurant rt ON mrt.restaurant = rt.id
        WHERE m.id = $1
        "#,
        meal_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    let food_source = match food_source_result {
        Ok(Some(row)) => match row.food_source_type.as_deref() {
            Some("recipe") if row.recipe_id.is_some() => Some(MealFoodSource::Recipe {
                recipe: Recipe {
                    id: row.recipe_id.unwrap(),
                    name: row.recipe_name.unwrap(),
                    ingredients: row.ingredients.unwrap(),
                    procedure: row.procedure.unwrap(),
                    cautions: row.cautions,
                },
                meal_type: row.meal_type.unwrap_or_else(|| "unknown".to_string()),
            }),
            Some("product") if row.product_id.is_some() => Some(MealFoodSource::Product {
                product: Product {
                    id: row.product_id.unwrap(),
                    name: row.product_name.unwrap(),
                },
                meal_type: row.meal_type.unwrap_or_else(|| "unknown".to_string()),
            }),
            Some("restaurant") if row.restaurant_id.is_some() => Some(MealFoodSource::Restaurant {
                restaurant: Restaurant {
                    id: row.restaurant_id.unwrap(),
                    name: row.restaurant_name.unwrap(),
                    location: row.location.unwrap(),
                    food_type: row.restaurant_type.unwrap(),
                    price: row.price,
                },
                meal_type: row.meal_type.unwrap_or_else(|| "unknown".to_string()),
            }),
            _ => None,
        },
        Ok(None) => None,
        Err(e) => {
            log::error!("Failed to fetch meal food source {}: {}", meal_id, e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch meal details"
            })));
        }
    };

    // Get people associated with this meal
    let people_result = sqlx::query_as::<_, People>(
        r#"
        SELECT p.id, p.name, p.notes
        FROM people p
        JOIN meal_people mp ON p.id = mp.people
        WHERE mp.meal = $1
        ORDER BY p.name
        "#,
    )
    .bind(meal_id)
    .fetch_all(pool.get_ref())
    .await;

    let people = match people_result {
        Ok(people) => people,
        Err(e) => {
            log::error!("Failed to fetch meal people {}: {}", meal_id, e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch meal details"
            })));
        }
    };

    let meal_detail = MealDetail {
        id: meal.id,
        date: meal.date,
        time: meal.time,
        notes: meal.notes,
        food_source,
        people,
    };

    Ok(HttpResponse::Ok().json(meal_detail))
}

#[derive(Debug, Deserialize)]
pub struct BatchDeleteMealsRequest {
    pub meal_ids: Vec<i32>,
}

async fn delete_meals_batch(
    pool: web::Data<PgPool>,
    request: web::Json<BatchDeleteMealsRequest>,
) -> Result<HttpResponse> {
    if request.meal_ids.is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "No meal IDs provided"
        })));
    }

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Failed to start transaction: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete meals"
            })));
        }
    };

    let mut deleted_count = 0;

    for meal_id in &request.meal_ids {
        // Check if meal exists
        let meal_exists = match sqlx::query!("SELECT id FROM meal WHERE id = $1", meal_id)
            .fetch_optional(&mut *tx)
            .await
        {
            Ok(Some(_)) => true,
            Ok(None) => {
                log::warn!("Meal ID {} not found, skipping", meal_id);
                continue;
            }
            Err(e) => {
                log::error!("Failed to check if meal {} exists: {}", meal_id, e);
                if let Err(rollback_err) = tx.rollback().await {
                    log::error!("Failed to rollback transaction: {}", rollback_err);
                }
                return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to delete meals"
                })));
            }
        };

        if !meal_exists {
            continue;
        }

        // Delete meal relationships
        let relationship_tables = [
            "meal_people",
            "meal_recipe",
            "meal_product",
            "meal_restaurant",
        ];

        for table in relationship_tables {
            if let Err(e) = sqlx::query(&format!("DELETE FROM {} WHERE meal = $1", table))
                .bind(meal_id)
                .execute(&mut *tx)
                .await
            {
                log::error!(
                    "Failed to delete {} relationships for meal {}: {}",
                    table,
                    meal_id,
                    e
                );
                if let Err(rollback_err) = tx.rollback().await {
                    log::error!("Failed to rollback transaction: {}", rollback_err);
                }
                return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to delete meals"
                })));
            }
        }

        // Delete the meal itself
        if let Err(e) = sqlx::query!("DELETE FROM meal WHERE id = $1", meal_id)
            .execute(&mut *tx)
            .await
        {
            log::error!("Failed to delete meal {}: {}", meal_id, e);
            if let Err(rollback_err) = tx.rollback().await {
                log::error!("Failed to rollback transaction: {}", rollback_err);
            }
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete meals"
            })));
        }

        deleted_count += 1;
    }

    // Commit the transaction
    if let Err(e) = tx.commit().await {
        log::error!("Failed to commit transaction: {}", e);
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to delete meals"
        })));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": format!("{} meals deleted successfully", deleted_count),
        "deleted_count": deleted_count
    })))
}
