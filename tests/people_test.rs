#[cfg(test)]
mod tests {
    use actix_web::{test, web, App};
    use sqlx::PgPool;
    use xnote::handlers::people;
    use serial_test::serial;
    
    struct TestContext {
        pool: PgPool,
        person1_id: i32,
        person2_id: i32,
        person3_id: i32,
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
        
        // Insert people
        let person1_id: i32 = sqlx::query_scalar!(
            "INSERT INTO people (name, notes) VALUES ($1, $2) RETURNING id",
            "Alice", Some("Test person 1")
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert person 1");
        
        let person2_id: i32 = sqlx::query_scalar!(
            "INSERT INTO people (name, notes) VALUES ($1, $2) RETURNING id", 
            "Bob", None::<String>
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert person 2");
        
        let person3_id: i32 = sqlx::query_scalar!(
            "INSERT INTO people (name, notes) VALUES ($1, $2) RETURNING id", 
            "Charlie", Some("Test person 3")
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert person 3");
        
        TestContext {
            pool,
            person1_id,
            person2_id,
            person3_id,
        }
    }
    
    async fn teardown_test_context(ctx: TestContext) {
        cleanup_database(&ctx.pool).await;
        ctx.pool.close().await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_get_people_list() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(people::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri("/people")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let people: Vec<xnote::models::people::People> = serde_json::from_slice(&body)
            .expect("Failed to deserialize people list");
            
        assert_eq!(people.len(), 3);
        
        // Check people are ordered by name
        assert_eq!(people[0].name, "Alice");
        assert_eq!(people[1].name, "Bob");
        assert_eq!(people[2].name, "Charlie");
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_get_person_by_id_alice() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(people::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri(&format!("/people/{}", ctx.person1_id))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let person: xnote::models::people::People = serde_json::from_slice(&body)
            .expect("Failed to deserialize person");
            
        assert_eq!(person.id, ctx.person1_id);
        assert_eq!(person.name, "Alice");
        assert_eq!(person.notes, Some("Test person 1".to_string()));
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_get_person_by_id_bob() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(people::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri(&format!("/people/{}", ctx.person2_id))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let person: xnote::models::people::People = serde_json::from_slice(&body)
            .expect("Failed to deserialize person");
            
        assert_eq!(person.id, ctx.person2_id);
        assert_eq!(person.name, "Bob");
        assert_eq!(person.notes, None);
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_get_person_by_id_not_found() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(people::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri("/people/99999")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
        
        let body = test::read_body(resp).await;
        let error_response: serde_json::Value = serde_json::from_slice(&body)
            .expect("Failed to deserialize error response");
        assert_eq!(error_response["error"], "Person not found");
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_create_person_placeholder() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(people::configure)
        ).await;
        
        let req = test::TestRequest::post()
            .uri("/people")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
        
        let body = test::read_body(resp).await;
        let response: serde_json::Value = serde_json::from_slice(&body)
            .expect("Failed to deserialize response");
        assert_eq!(response["message"], "Create person - TODO: implement");
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_update_person_placeholder() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(people::configure)
        ).await;
        
        let req = test::TestRequest::put()
            .uri(&format!("/people/{}", ctx.person1_id))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let response: serde_json::Value = serde_json::from_slice(&body)
            .expect("Failed to deserialize response");
        assert_eq!(response["message"], "Update person - TODO: implement");
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_delete_person_placeholder() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(people::configure)
        ).await;
        
        let req = test::TestRequest::delete()
            .uri(&format!("/people/{}", ctx.person1_id))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let response: serde_json::Value = serde_json::from_slice(&body)
            .expect("Failed to deserialize response");
        assert_eq!(response["message"], "Delete person - TODO: implement");
        
        teardown_test_context(ctx).await;
    }
}