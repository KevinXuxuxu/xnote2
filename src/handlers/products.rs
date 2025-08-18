use actix_web::{web, HttpResponse, Result};
use sqlx::PgPool;
use crate::models::product::{Product, CreateProduct, UpdateProduct};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/products")
            .route(web::get().to(get_products))
            .route(web::post().to(create_product))
    )
    .service(
        web::resource("/products/{id}")
            .route(web::get().to(get_product))
            .route(web::put().to(update_product))
            .route(web::delete().to(delete_product))
    );
}

async fn get_products(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    match sqlx::query_as::<_, Product>(
        "SELECT id, name FROM product ORDER BY id"
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(products) => Ok(HttpResponse::Ok().json(products)),
        Err(e) => {
            log::error!("Failed to fetch products: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch products"
            })))
        }
    }
}

async fn create_product(
    pool: web::Data<PgPool>,
    product_data: web::Json<CreateProduct>
) -> Result<HttpResponse> {
    match sqlx::query_as::<_, Product>(
        r#"
        INSERT INTO product (name)
        VALUES ($1)
        RETURNING id, name
        "#
    )
    .bind(&product_data.name)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(product) => Ok(HttpResponse::Created().json(product)),
        Err(e) => {
            log::error!("Failed to create product: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create product"
            })))
        }
    }
}

async fn get_product(pool: web::Data<PgPool>, path: web::Path<i32>) -> Result<HttpResponse> {
    let product_id = path.into_inner();
    
    match sqlx::query_as::<_, Product>(
        "SELECT id, name FROM product WHERE id = $1"
    )
    .bind(product_id)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(product)) => Ok(HttpResponse::Ok().json(product)),
        Ok(None) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Product not found"
        }))),
        Err(e) => {
            log::error!("Failed to fetch product {}: {}", product_id, e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch product"
            })))
        }
    }
}

async fn update_product(
    pool: web::Data<PgPool>, 
    path: web::Path<i32>,
    product_data: web::Json<UpdateProduct>
) -> Result<HttpResponse> {
    let product_id = path.into_inner();
    
    if product_data.name.is_none() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "No fields to update"
        })));
    }
    
    match sqlx::query_as::<_, Product>(
        "UPDATE product SET name = $2 WHERE id = $1 RETURNING id, name"
    )
    .bind(product_id)
    .bind(&product_data.name)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(Some(product)) => Ok(HttpResponse::Ok().json(product)),
        Ok(None) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Product not found"
        }))),
        Err(e) => {
            log::error!("Failed to update product {}: {}", product_id, e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update product"
            })))
        }
    }
}

async fn delete_product(pool: web::Data<PgPool>, path: web::Path<i32>) -> Result<HttpResponse> {
    let product_id = path.into_inner();
    
    // First check if product exists
    let product_exists = sqlx::query!(
        "SELECT id FROM product WHERE id = $1",
        product_id
    )
    .fetch_optional(pool.get_ref())
    .await;
    
    match product_exists {
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Product not found"
            })));
        },
        Err(e) => {
            log::error!("Failed to check product existence {}: {}", product_id, e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete product"
            })));
        },
        Ok(Some(_)) => {} // Product exists, continue
    }
    
    // Check if product is referenced in meals
    let meal_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM meal_product WHERE product = $1",
        product_id
    )
    .fetch_one(pool.get_ref())
    .await;
    
    match meal_count {
        Ok(result) => {
            if result.count.unwrap_or(0) > 0 {
                return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": format!("Cannot delete product: it is referenced in {} meal(s). Please delete those meals first.", result.count.unwrap_or(0))
                })));
            }
        },
        Err(e) => {
            log::error!("Failed to check meal references for product {}: {}", product_id, e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete product"
            })));
        }
    }
    
    // Now safe to delete the product
    match sqlx::query!(
        "DELETE FROM product WHERE id = $1",
        product_id
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "message": "Product deleted successfully"
                })))
            } else {
                // This shouldn't happen since we checked existence above
                Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Product not found"
                })))
            }
        },
        Err(e) => {
            log::error!("Failed to delete product {}: {}", product_id, e);
            
            // Check if it's a foreign key constraint error
            let error_message = if e.to_string().contains("foreign key") {
                "Cannot delete product: it is still referenced by other records"
            } else {
                "Failed to delete product"
            };
            
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": error_message
            })))
        }
    }
}