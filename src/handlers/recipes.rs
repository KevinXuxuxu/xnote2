use actix_web::{web, HttpResponse, Result};
use sqlx::PgPool;
use crate::models::recipe::{Recipe, CreateRecipe, UpdateRecipe};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/recipes")
            .route(web::get().to(get_recipes))
            .route(web::post().to(create_recipe))
    )
    .service(
        web::resource("/recipes/{id}")
            .route(web::get().to(get_recipe))
            .route(web::put().to(update_recipe))
            .route(web::delete().to(delete_recipe))
    );
}

async fn get_recipes(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    match sqlx::query_as::<_, Recipe>(
        "SELECT id, name, ingredients, procedure, cautions FROM recipe ORDER BY id"
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(recipes) => Ok(HttpResponse::Ok().json(recipes)),
        Err(e) => {
            log::error!("Failed to fetch recipes: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch recipes"
            })))
        }
    }
}

async fn create_recipe(
    pool: web::Data<PgPool>,
    recipe_data: web::Json<CreateRecipe>
) -> Result<HttpResponse> {
    match sqlx::query_as::<_, Recipe>(
        r#"
        INSERT INTO recipe (name, ingredients, procedure, cautions)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, ingredients, procedure, cautions
        "#
    )
    .bind(&recipe_data.name)
    .bind(&recipe_data.ingredients)
    .bind(&recipe_data.procedure)
    .bind(&recipe_data.cautions)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(recipe) => Ok(HttpResponse::Created().json(recipe)),
        Err(e) => {
            log::error!("Failed to create recipe: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create recipe"
            })))
        }
    }
}

async fn get_recipe(pool: web::Data<PgPool>, path: web::Path<i32>) -> Result<HttpResponse> {
    let recipe_id = path.into_inner();
    
    match sqlx::query_as::<_, Recipe>(
        "SELECT id, name, ingredients, procedure, cautions FROM recipe WHERE id = $1"
    )
    .bind(recipe_id)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(recipe)) => Ok(HttpResponse::Ok().json(recipe)),
        Ok(None) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Recipe not found"
        }))),
        Err(e) => {
            log::error!("Failed to fetch recipe {}: {}", recipe_id, e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch recipe"
            })))
        }
    }
}

async fn update_recipe(
    pool: web::Data<PgPool>, 
    path: web::Path<i32>,
    recipe_data: web::Json<UpdateRecipe>
) -> Result<HttpResponse> {
    let recipe_id = path.into_inner();
    
    // Build dynamic update query
    let mut query_parts = Vec::new();
    let mut param_index = 2; // Start from $2 since $1 is the ID
    
    if recipe_data.name.is_some() {
        query_parts.push(format!("name = ${}", param_index));
        param_index += 1;
    }
    if recipe_data.ingredients.is_some() {
        query_parts.push(format!("ingredients = ${}", param_index));
        param_index += 1;
    }
    if recipe_data.procedure.is_some() {
        query_parts.push(format!("procedure = ${}", param_index));
        param_index += 1;
    }
    if recipe_data.cautions.is_some() {
        query_parts.push(format!("cautions = ${}", param_index));
    }
    
    if query_parts.is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "No fields to update"
        })));
    }
    
    let query = format!(
        "UPDATE recipe SET {} WHERE id = $1 RETURNING id, name, ingredients, procedure, cautions",
        query_parts.join(", ")
    );
    
    let mut query_builder = sqlx::query_as::<_, Recipe>(&query).bind(recipe_id);
    
    // Bind parameters in the same order
    if let Some(ref name) = recipe_data.name {
        query_builder = query_builder.bind(name);
    }
    if let Some(ref ingredients) = recipe_data.ingredients {
        query_builder = query_builder.bind(ingredients);
    }
    if let Some(ref procedure) = recipe_data.procedure {
        query_builder = query_builder.bind(procedure);
    }
    if recipe_data.cautions.is_some() {
        query_builder = query_builder.bind(&recipe_data.cautions);
    }
    
    match query_builder.fetch_optional(pool.get_ref()).await {
        Ok(Some(recipe)) => Ok(HttpResponse::Ok().json(recipe)),
        Ok(None) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Recipe not found"
        }))),
        Err(e) => {
            log::error!("Failed to update recipe {}: {}", recipe_id, e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update recipe"
            })))
        }
    }
}

async fn delete_recipe(pool: web::Data<PgPool>, path: web::Path<i32>) -> Result<HttpResponse> {
    let recipe_id = path.into_inner();
    
    // First check if recipe exists
    let recipe_exists = sqlx::query!(
        "SELECT id FROM recipe WHERE id = $1",
        recipe_id
    )
    .fetch_optional(pool.get_ref())
    .await;
    
    match recipe_exists {
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Recipe not found"
            })));
        },
        Err(e) => {
            log::error!("Failed to check recipe existence {}: {}", recipe_id, e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete recipe"
            })));
        },
        Ok(Some(_)) => {} // Recipe exists, continue
    }
    
    // Check if recipe is referenced in meals
    let meal_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM meal_recipe WHERE recipe = $1",
        recipe_id
    )
    .fetch_one(pool.get_ref())
    .await;
    
    match meal_count {
        Ok(result) => {
            if result.count.unwrap_or(0) > 0 {
                return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Cannot delete recipe: it is referenced in {} meal(s). Please delete those meals first.", result.count.unwrap_or(0))
                })));
            }
        },
        Err(e) => {
            log::error!("Failed to check meal references for recipe {}: {}", recipe_id, e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete recipe"
            })));
        }
    }
    
    // Now safe to delete the recipe
    match sqlx::query!(
        "DELETE FROM recipe WHERE id = $1",
        recipe_id
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "message": "Recipe deleted successfully"
                })))
            } else {
                // This shouldn't happen since we checked existence above
                Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Recipe not found"
                })))
            }
        },
        Err(e) => {
            log::error!("Failed to delete recipe {}: {}", recipe_id, e);
            
            // Check if it's a foreign key constraint error
            let error_message = if e.to_string().contains("foreign key") {
                "Cannot delete recipe: it is still referenced by other records"
            } else {
                "Failed to delete recipe"
            };
            
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": error_message
            })))
        }
    }
}