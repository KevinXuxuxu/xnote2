#[cfg(test)]
mod tests {
    use actix_web::{test, web, App};
    use sqlx::PgPool;
    use xnote::handlers::events;
    use xnote::models::detail::{EventDetail, ActivityDetail};
    use serial_test::serial;
    
    struct TestContext {
        pool: PgPool,
        event1_id: i32,    // Event with people
        event2_id: i32,    // Event with one person  
        event3_id: i32,    // Event with no people
        activity1_id: i32,
        activity2_id: i32,
        activity3_id: i32,
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
        
        // Insert activities
        let activity1_id: i32 = sqlx::query_scalar!(
            "INSERT INTO activity (name, type) VALUES ($1, $2) RETURNING id",
            "Running", "sport"
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert activity 1");
        
        let activity2_id: i32 = sqlx::query_scalar!(
            "INSERT INTO activity (name, type) VALUES ($1, $2) RETURNING id",
            "Coding", "side project"
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert activity 2");
        
        let activity3_id: i32 = sqlx::query_scalar!(
            "INSERT INTO activity (name, type) VALUES ($1, $2) RETURNING id",
            "Cleaning", "chore"
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert activity 3");
        
        // Insert events
        let event1_id: i32 = sqlx::query_scalar!(
            "INSERT INTO event (date, activity, measure, location, notes) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            activity1_id,
            Some("5 miles"),
            Some("Seattle Downtown"),
            Some("Great workout")
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert event 1");
        
        let event2_id: i32 = sqlx::query_scalar!(
            "INSERT INTO event (date, activity, measure, location, notes) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
            activity2_id,
            Some("2 hours"),
            Some("SLU"),
            None::<String>
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert event 2");
        
        let event3_id: i32 = sqlx::query_scalar!(
            "INSERT INTO event (date, activity, measure, location, notes) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 17).unwrap(),
            activity3_id,
            None::<String>,
            None::<String>,
            Some("House cleaning")
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert event 3");
        
        // Link people to events
        sqlx::query!(
            "INSERT INTO event_people (event, people) VALUES ($1, $2)",
            event1_id, person1_id
        )
        .execute(&pool)
        .await
        .expect("Failed to link person 1 to event 1");
        
        sqlx::query!(
            "INSERT INTO event_people (event, people) VALUES ($1, $2)",
            event1_id, person2_id
        )
        .execute(&pool)
        .await
        .expect("Failed to link person 2 to event 1");
        
        sqlx::query!(
            "INSERT INTO event_people (event, people) VALUES ($1, $2)",
            event2_id, person1_id
        )
        .execute(&pool)
        .await
        .expect("Failed to link person 1 to event 2");
        
        TestContext {
            pool,
            event1_id,
            event2_id,
            event3_id,
            activity1_id,
            activity2_id,
            activity3_id,
        }
    }
    
    async fn teardown_test_context(ctx: TestContext) {
        cleanup_database(&ctx.pool).await;
        ctx.pool.close().await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_event_details_with_people() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(events::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri(&format!("/events/{}/details", ctx.event1_id))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let event_detail: EventDetail = serde_json::from_slice(&body)
            .expect("Failed to deserialize event detail");
            
        assert_eq!(event_detail.id, ctx.event1_id);
        assert_eq!(event_detail.activity.id, ctx.activity1_id);
        assert_eq!(event_detail.activity.name, "Running");
        assert_eq!(event_detail.activity.activity_type, "sport");
        assert_eq!(event_detail.measure, Some("5 miles".to_string()));
        assert_eq!(event_detail.location, Some("Seattle Downtown".to_string()));
        assert_eq!(event_detail.notes, Some("Great workout".to_string()));
        assert_eq!(event_detail.people.len(), 2);
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_event_details_with_one_person() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(events::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri(&format!("/events/{}/details", ctx.event2_id))
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let event_detail: EventDetail = serde_json::from_slice(&body)
            .expect("Failed to deserialize event detail");
            
        assert_eq!(event_detail.id, ctx.event2_id);
        assert_eq!(event_detail.activity.id, ctx.activity2_id);
        assert_eq!(event_detail.activity.name, "Coding");
        assert_eq!(event_detail.activity.activity_type, "side project");
        assert_eq!(event_detail.measure, Some("2 hours".to_string()));
        assert_eq!(event_detail.location, Some("SLU".to_string()));
        assert_eq!(event_detail.notes, None);
        assert_eq!(event_detail.people.len(), 1);
        assert_eq!(event_detail.people[0].name, "Alice");
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_event_details_no_people() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(events::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri(&format!("/events/{}/details", ctx.event3_id))
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let event_detail: EventDetail = serde_json::from_slice(&body)
            .expect("Failed to deserialize event detail");
            
        assert_eq!(event_detail.id, ctx.event3_id);
        assert_eq!(event_detail.activity.id, ctx.activity3_id);
        assert_eq!(event_detail.activity.name, "Cleaning");
        assert_eq!(event_detail.activity.activity_type, "chore");
        assert_eq!(event_detail.measure, None);
        assert_eq!(event_detail.location, None);
        assert_eq!(event_detail.notes, Some("House cleaning".to_string()));
        assert_eq!(event_detail.people.len(), 0);
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_event_details_not_found() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(events::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri("/events/99999/details")
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
        
        let body = test::read_body(resp).await;
        let error_response: serde_json::Value = serde_json::from_slice(&body)
            .expect("Failed to deserialize error response");
        assert_eq!(error_response["error"], "Event not found");
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_get_events_list() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(events::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri("/events")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let events: Vec<xnote::models::event::Event> = serde_json::from_slice(&body)
            .expect("Failed to deserialize events list");
            
        assert_eq!(events.len(), 3);
        
        // Check events are ordered by date DESC
        assert_eq!(events[0].id, ctx.event3_id); // 2024-01-17
        assert_eq!(events[1].id, ctx.event2_id); // 2024-01-16
        assert_eq!(events[2].id, ctx.event1_id); // 2024-01-15
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_get_event_by_id_running() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(events::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri(&format!("/events/{}", ctx.event1_id))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let event: xnote::models::event::Event = serde_json::from_slice(&body)
            .expect("Failed to deserialize event");
            
        assert_eq!(event.id, ctx.event1_id);
        assert_eq!(event.activity, ctx.activity1_id);
        assert_eq!(event.measure, Some("5 miles".to_string()));
        assert_eq!(event.location, Some("Seattle Downtown".to_string()));
        assert_eq!(event.notes, Some("Great workout".to_string()));
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_get_event_by_id_coding() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(events::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri(&format!("/events/{}", ctx.event2_id))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let event: xnote::models::event::Event = serde_json::from_slice(&body)
            .expect("Failed to deserialize event");
            
        assert_eq!(event.id, ctx.event2_id);
        assert_eq!(event.activity, ctx.activity2_id);
        assert_eq!(event.measure, Some("2 hours".to_string()));
        assert_eq!(event.location, Some("SLU".to_string()));
        assert_eq!(event.notes, None);
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_get_event_by_id_cleaning() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(events::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri(&format!("/events/{}", ctx.event3_id))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let event: xnote::models::event::Event = serde_json::from_slice(&body)
            .expect("Failed to deserialize event");
            
        assert_eq!(event.id, ctx.event3_id);
        assert_eq!(event.activity, ctx.activity3_id);
        assert_eq!(event.measure, None);
        assert_eq!(event.location, None);
        assert_eq!(event.notes, Some("House cleaning".to_string()));
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_get_event_by_id_not_found() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(events::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri("/events/99999")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
        
        let body = test::read_body(resp).await;
        let error_response: serde_json::Value = serde_json::from_slice(&body)
            .expect("Failed to deserialize error response");
        assert_eq!(error_response["error"], "Event not found");
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_create_event_placeholder() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(events::configure)
        ).await;
        
        let req = test::TestRequest::post()
            .uri("/events")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
        
        let body = test::read_body(resp).await;
        let response: serde_json::Value = serde_json::from_slice(&body)
            .expect("Failed to deserialize response");
        assert_eq!(response["message"], "Create event - TODO: implement");
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_update_event_placeholder() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(events::configure)
        ).await;
        
        let req = test::TestRequest::put()
            .uri(&format!("/events/{}", ctx.event1_id))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let response: serde_json::Value = serde_json::from_slice(&body)
            .expect("Failed to deserialize response");
        assert_eq!(response["message"], "Update event - TODO: implement");
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_delete_event_placeholder() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(events::configure)
        ).await;
        
        let req = test::TestRequest::delete()
            .uri(&format!("/events/{}", ctx.event1_id))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let response: serde_json::Value = serde_json::from_slice(&body)
            .expect("Failed to deserialize response");
        assert_eq!(response["message"], "Delete event - TODO: implement");
        
        teardown_test_context(ctx).await;
    }
}