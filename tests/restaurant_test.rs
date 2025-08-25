#[cfg(test)]
mod tests {
    use actix_web::{test, web, App};
    use serial_test::serial;
    use sqlx::PgPool;
    use xnote::handlers::restaurants;

    struct TestContext {
        pool: PgPool,
        restaurant1_id: i32,
        restaurant2_id: i32,
        restaurant3_id: i32,
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
            "drink_people",
            "drink",
            "drink_option",
            "event_people",
            "event",
            "activity",
            "activity_type",
            "meal_people",
            "meal_restaurant",
            "meal_product",
            "meal_recipe",
            "meal",
            "meal_time",
            "meal_type",
            "people",
            "restaurant",
            "product",
            "recipe",
            "location",
            "food_type",
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

        // Insert restaurants
        let restaurant1_id: i32 = sqlx::query_scalar!(
            "INSERT INTO restaurant (name, location, type, price) VALUES ($1, $2, $3, $4) RETURNING id",
            "Pasta Palace", "Seattle Downtown", "Italian", Some(25.50)
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert restaurant 1");

        let restaurant2_id: i32 = sqlx::query_scalar!(
            "INSERT INTO restaurant (name, location, type, price) VALUES ($1, $2, $3, $4) RETURNING id",
            "Burger Joint", "Capitol Hill", "fast food", None::<f32>
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert restaurant 2");

        let restaurant3_id: i32 = sqlx::query_scalar!(
            "INSERT INTO restaurant (name, location, type, price) VALUES ($1, $2, $3, $4) RETURNING id",
            "Taco Truck", "Ballard", "mexican", Some(12.75)
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert restaurant 3");

        TestContext {
            pool,
            restaurant1_id,
            restaurant2_id,
            restaurant3_id,
        }
    }

    async fn teardown_test_context(ctx: TestContext) {
        cleanup_database(&ctx.pool).await;
        ctx.pool.close().await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_restaurants_list() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(restaurants::configure),
        )
        .await;

        let req = test::TestRequest::get().uri("/restaurants").to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let restaurants: Vec<xnote::models::restaurant::Restaurant> =
            serde_json::from_slice(&body).expect("Failed to deserialize restaurants list");

        assert_eq!(restaurants.len(), 3);

        // Check restaurants are ordered by name
        assert_eq!(restaurants[0].name, "Burger Joint");
        assert_eq!(restaurants[1].name, "Pasta Palace");
        assert_eq!(restaurants[2].name, "Taco Truck");

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_restaurant_by_id_pasta() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(restaurants::configure),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/restaurants/{}", ctx.restaurant1_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let restaurant: xnote::models::restaurant::Restaurant =
            serde_json::from_slice(&body).expect("Failed to deserialize restaurant");

        assert_eq!(restaurant.id, ctx.restaurant1_id);
        assert_eq!(restaurant.name, "Pasta Palace");
        assert_eq!(restaurant.location, "Seattle Downtown");
        assert_eq!(restaurant.food_type, "Italian");
        assert_eq!(restaurant.price, Some(25.50));

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_restaurant_by_id_burger() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(restaurants::configure),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/restaurants/{}", ctx.restaurant2_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let restaurant: xnote::models::restaurant::Restaurant =
            serde_json::from_slice(&body).expect("Failed to deserialize restaurant");

        assert_eq!(restaurant.id, ctx.restaurant2_id);
        assert_eq!(restaurant.name, "Burger Joint");
        assert_eq!(restaurant.location, "Capitol Hill");
        assert_eq!(restaurant.food_type, "fast food");
        assert_eq!(restaurant.price, None);

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_restaurant_by_id_not_found() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(restaurants::configure),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/restaurants/99999")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        let body = test::read_body(resp).await;
        let error_response: serde_json::Value =
            serde_json::from_slice(&body).expect("Failed to deserialize error response");
        assert_eq!(error_response["error"], "Restaurant not found");

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_create_restaurant_placeholder() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(restaurants::configure),
        )
        .await;

        let req = test::TestRequest::post().uri("/restaurants").to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        let body = test::read_body(resp).await;
        let response: serde_json::Value =
            serde_json::from_slice(&body).expect("Failed to deserialize response");
        assert_eq!(response["message"], "Create restaurant - TODO: implement");

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_update_restaurant_placeholder() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(restaurants::configure),
        )
        .await;

        let req = test::TestRequest::put()
            .uri(&format!("/restaurants/{}", ctx.restaurant1_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let response: serde_json::Value =
            serde_json::from_slice(&body).expect("Failed to deserialize response");
        assert_eq!(response["message"], "Update restaurant - TODO: implement");

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_delete_restaurant_placeholder() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(restaurants::configure),
        )
        .await;

        let req = test::TestRequest::delete()
            .uri(&format!("/restaurants/{}", ctx.restaurant1_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let response: serde_json::Value =
            serde_json::from_slice(&body).expect("Failed to deserialize response");
        assert_eq!(response["message"], "Delete restaurant - TODO: implement");

        teardown_test_context(ctx).await;
    }
}
