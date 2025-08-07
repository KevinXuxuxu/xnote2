#[cfg(test)]
mod tests {
    use actix_web::{test, web, App};
    use sqlx::PgPool;
    use xnote::handlers::products;
    use serial_test::serial;
    
    struct TestContext {
        pool: PgPool,
        product1_id: i32,
        product2_id: i32,
        product3_id: i32,
    }
    
    async fn create_test_database_pool() -> PgPool {
        dotenv::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set")
            .replace("/xnote", "/xnote_test");
        PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }
    
    async fn create_schema(pool: &PgPool) {
        let schema_sql = include_str!("../init.sql");
        sqlx::raw_sql(schema_sql)
            .execute(pool)
            .await
            .expect("Failed to create schema");
    }
    
    async fn cleanup_database(pool: &PgPool) {
        let tables = vec![
            "drink_people", "drink", "drink_option",
            "event_people", "event", "activity", "activity_type",
            "meal_people", "meal_restaurant", "meal_product", "meal_recipe", "meal",
            "meal_time", "meal_type", "people", "restaurant", "product", "recipe",
            "location", "food_type"
        ];
        
        for table in tables {
            let query = format!("DROP TABLE IF EXISTS {} CASCADE", table);
            sqlx::query(&query)
                .execute(pool)
                .await
                .expect(&format!("Failed to drop table {}", table));
        }
    }
    
    async fn setup_test_context() -> TestContext {
        let pool = create_test_database_pool().await;
        create_schema(&pool).await;
        
        // Insert products
        let product1_id: i32 = sqlx::query_scalar!(
            "INSERT INTO product (name) VALUES ($1) RETURNING id",
            "Apple"
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert product 1");
        
        let product2_id: i32 = sqlx::query_scalar!(
            "INSERT INTO product (name) VALUES ($1) RETURNING id",
            "Banana"
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert product 2");
        
        let product3_id: i32 = sqlx::query_scalar!(
            "INSERT INTO product (name) VALUES ($1) RETURNING id",
            "Orange"
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert product 3");
        
        TestContext {
            pool,
            product1_id,
            product2_id,
            product3_id,
        }
    }
    
    async fn teardown_test_context(ctx: TestContext) {
        cleanup_database(&ctx.pool).await;
        ctx.pool.close().await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_get_products_list() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(products::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri("/products")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let products: Vec<xnote::models::product::Product> = serde_json::from_slice(&body)
            .expect("Failed to deserialize products list");
            
        assert_eq!(products.len(), 3);
        
        // Check products are ordered by name
        assert_eq!(products[0].name, "Apple");
        assert_eq!(products[1].name, "Banana");
        assert_eq!(products[2].name, "Orange");
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_get_product_by_id_apple() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(products::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri(&format!("/products/{}", ctx.product1_id))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let product: xnote::models::product::Product = serde_json::from_slice(&body)
            .expect("Failed to deserialize product");
            
        assert_eq!(product.id, ctx.product1_id);
        assert_eq!(product.name, "Apple");
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_get_product_by_id_banana() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(products::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri(&format!("/products/{}", ctx.product2_id))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let product: xnote::models::product::Product = serde_json::from_slice(&body)
            .expect("Failed to deserialize product");
            
        assert_eq!(product.id, ctx.product2_id);
        assert_eq!(product.name, "Banana");
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_get_product_by_id_not_found() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(products::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri("/products/99999")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
        
        let body = test::read_body(resp).await;
        let error_response: serde_json::Value = serde_json::from_slice(&body)
            .expect("Failed to deserialize error response");
        assert_eq!(error_response["error"], "Product not found");
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_create_product_placeholder() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(products::configure)
        ).await;
        
        let req = test::TestRequest::post()
            .uri("/products")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
        
        let body = test::read_body(resp).await;
        let response: serde_json::Value = serde_json::from_slice(&body)
            .expect("Failed to deserialize response");
        assert_eq!(response["message"], "Create product - TODO: implement");
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_update_product_placeholder() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(products::configure)
        ).await;
        
        let req = test::TestRequest::put()
            .uri(&format!("/products/{}", ctx.product1_id))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let response: serde_json::Value = serde_json::from_slice(&body)
            .expect("Failed to deserialize response");
        assert_eq!(response["message"], "Update product - TODO: implement");
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_delete_product_placeholder() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(products::configure)
        ).await;
        
        let req = test::TestRequest::delete()
            .uri(&format!("/products/{}", ctx.product1_id))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let response: serde_json::Value = serde_json::from_slice(&body)
            .expect("Failed to deserialize response");
        assert_eq!(response["message"], "Delete product - TODO: implement");
        
        teardown_test_context(ctx).await;
    }
}