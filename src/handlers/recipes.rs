use actix_web::{web, HttpResponse, Result};
use sqlx::PgPool;
use crate::models::recipe::Recipe;

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
        "SELECT id, name, ingredients, procedure, cautions FROM recipe ORDER BY name"
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

async fn create_recipe(_pool: web::Data<PgPool>) -> Result<HttpResponse> {
    Ok(HttpResponse::Created().json(serde_json::json!({
        "message": "Create recipe - TODO: implement"
    })))
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

async fn update_recipe(_pool: web::Data<PgPool>, _path: web::Path<i32>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Update recipe - TODO: implement"
    })))
}

async fn delete_recipe(_pool: web::Data<PgPool>, _path: web::Path<i32>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Delete recipe - TODO: implement"
    })))
}