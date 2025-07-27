use actix_web::{web, HttpResponse, Result};
use sqlx::PgPool;
use crate::models::product::Product;

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
        "SELECT id, name FROM product ORDER BY name"
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

async fn create_product(_pool: web::Data<PgPool>) -> Result<HttpResponse> {
    Ok(HttpResponse::Created().json(serde_json::json!({
        "message": "Create product - TODO: implement"
    })))
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

async fn update_product(_pool: web::Data<PgPool>, _path: web::Path<i32>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Update product - TODO: implement"
    })))
}

async fn delete_product(_pool: web::Data<PgPool>, _path: web::Path<i32>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Delete product - TODO: implement"
    })))
}