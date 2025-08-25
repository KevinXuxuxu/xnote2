#[cfg(test)]
mod tests {
    use actix_web::{test, web, App};
    use serial_test::serial;
    use sqlx::PgPool;
    use xnote::handlers::drinks;
    use xnote::models::detail::DrinkDetail;

    struct TestContext {
        pool: PgPool,
        drink1_id: i32, // Drink with people
        drink2_id: i32, // Drink with one person
        drink3_id: i32, // Drink with no people
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

        // Insert people
        let person1_id: i32 = sqlx::query_scalar!(
            "INSERT INTO people (name, notes) VALUES ($1, $2) RETURNING id",
            "Alice",
            Some("Test person 1")
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert person 1");

        let person2_id: i32 = sqlx::query_scalar!(
            "INSERT INTO people (name, notes) VALUES ($1, $2) RETURNING id",
            "Bob",
            None::<String>
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert person 2");

        // Insert drinks
        let drink1_id: i32 = sqlx::query_scalar!(
            "INSERT INTO drink (name, date) VALUES ($1, $2) RETURNING id",
            "Sip House - Ube Latte",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert drink 1");

        let drink2_id: i32 = sqlx::query_scalar!(
            "INSERT INTO drink (name, date) VALUES ($1, $2) RETURNING id",
            "自己做的latte",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 16).unwrap()
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert drink 2");

        let drink3_id: i32 = sqlx::query_scalar!(
            "INSERT INTO drink (name, date) VALUES ($1, $2) RETURNING id",
            "吃茶三千",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 17).unwrap()
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert drink 3");

        // Link people to drinks
        sqlx::query!(
            "INSERT INTO drink_people (drink, people) VALUES ($1, $2)",
            drink1_id,
            person1_id
        )
        .execute(&pool)
        .await
        .expect("Failed to link person 1 to drink 1");

        sqlx::query!(
            "INSERT INTO drink_people (drink, people) VALUES ($1, $2)",
            drink1_id,
            person2_id
        )
        .execute(&pool)
        .await
        .expect("Failed to link person 2 to drink 1");

        sqlx::query!(
            "INSERT INTO drink_people (drink, people) VALUES ($1, $2)",
            drink2_id,
            person1_id
        )
        .execute(&pool)
        .await
        .expect("Failed to link person 1 to drink 2");

        TestContext {
            pool,
            drink1_id,
            drink2_id,
            drink3_id,
        }
    }

    async fn teardown_test_context(ctx: TestContext) {
        cleanup_database(&ctx.pool).await;
        ctx.pool.close().await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_drink_details_with_people() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(drinks::configure),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/drinks/{}/details", ctx.drink1_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let drink_detail: DrinkDetail =
            serde_json::from_slice(&body).expect("Failed to deserialize drink detail");

        assert_eq!(drink_detail.id, ctx.drink1_id);
        assert_eq!(drink_detail.name, "Sip House - Ube Latte");
        assert_eq!(drink_detail.people.len(), 2);

        // Check people are sorted by name
        let alice_found = drink_detail.people.iter().any(|p| p.name == "Alice");
        let bob_found = drink_detail.people.iter().any(|p| p.name == "Bob");
        assert!(alice_found);
        assert!(bob_found);

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_drink_details_with_one_person() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(drinks::configure),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/drinks/{}/details", ctx.drink2_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let drink_detail: DrinkDetail =
            serde_json::from_slice(&body).expect("Failed to deserialize drink detail");

        assert_eq!(drink_detail.id, ctx.drink2_id);
        assert_eq!(drink_detail.name, "自己做的latte");
        assert_eq!(drink_detail.people.len(), 1);
        assert_eq!(drink_detail.people[0].name, "Alice");

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_drink_details_no_people() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(drinks::configure),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/drinks/{}/details", ctx.drink3_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let drink_detail: DrinkDetail =
            serde_json::from_slice(&body).expect("Failed to deserialize drink detail");

        assert_eq!(drink_detail.id, ctx.drink3_id);
        assert_eq!(drink_detail.name, "吃茶三千");
        assert_eq!(drink_detail.people.len(), 0);

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_drink_details_not_found() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(drinks::configure),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/drinks/99999/details")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        let body = test::read_body(resp).await;
        let error_response: serde_json::Value =
            serde_json::from_slice(&body).expect("Failed to deserialize error response");
        assert_eq!(error_response["error"], "Drink not found");

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_drinks_list() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(drinks::configure),
        )
        .await;

        let req = test::TestRequest::get().uri("/drinks").to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let drinks: Vec<xnote::models::drink::Drink> =
            serde_json::from_slice(&body).expect("Failed to deserialize drinks list");

        assert_eq!(drinks.len(), 3);

        // Check drinks are ordered by date DESC
        assert_eq!(drinks[0].id, ctx.drink3_id); // 2024-01-17 吃茶三千
        assert_eq!(drinks[1].id, ctx.drink2_id); // 2024-01-16 自己做的latte
        assert_eq!(drinks[2].id, ctx.drink1_id); // 2024-01-15 Sip House - Ube Latte

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_drink_by_id_ube_latte() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(drinks::configure),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/drinks/{}", ctx.drink1_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let drink: xnote::models::drink::Drink =
            serde_json::from_slice(&body).expect("Failed to deserialize drink");

        assert_eq!(drink.id, ctx.drink1_id);
        assert_eq!(drink.name, "Sip House - Ube Latte");
        assert_eq!(
            drink.date,
            chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()
        );

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_drink_by_id_homemade_latte() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(drinks::configure),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/drinks/{}", ctx.drink2_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let drink: xnote::models::drink::Drink =
            serde_json::from_slice(&body).expect("Failed to deserialize drink");

        assert_eq!(drink.id, ctx.drink2_id);
        assert_eq!(drink.name, "自己做的latte");
        assert_eq!(
            drink.date,
            chrono::NaiveDate::from_ymd_opt(2024, 1, 16).unwrap()
        );

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_drink_by_id_tea_shop() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(drinks::configure),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/drinks/{}", ctx.drink3_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let drink: xnote::models::drink::Drink =
            serde_json::from_slice(&body).expect("Failed to deserialize drink");

        assert_eq!(drink.id, ctx.drink3_id);
        assert_eq!(drink.name, "吃茶三千");
        assert_eq!(
            drink.date,
            chrono::NaiveDate::from_ymd_opt(2024, 1, 17).unwrap()
        );

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_drink_by_id_not_found() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(drinks::configure),
        )
        .await;

        let req = test::TestRequest::get().uri("/drinks/99999").to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        let body = test::read_body(resp).await;
        let error_response: serde_json::Value =
            serde_json::from_slice(&body).expect("Failed to deserialize error response");
        assert_eq!(error_response["error"], "Drink not found");

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_create_drink_placeholder() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(drinks::configure),
        )
        .await;

        let req = test::TestRequest::post().uri("/drinks").to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        let body = test::read_body(resp).await;
        let response: serde_json::Value =
            serde_json::from_slice(&body).expect("Failed to deserialize response");
        assert_eq!(response["message"], "Create drink - TODO: implement");

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_update_drink_placeholder() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(drinks::configure),
        )
        .await;

        let req = test::TestRequest::put()
            .uri(&format!("/drinks/{}", ctx.drink1_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let response: serde_json::Value =
            serde_json::from_slice(&body).expect("Failed to deserialize response");
        assert_eq!(response["message"], "Update drink - TODO: implement");

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_delete_drink_placeholder() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(drinks::configure),
        )
        .await;

        let req = test::TestRequest::delete()
            .uri(&format!("/drinks/{}", ctx.drink1_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let response: serde_json::Value =
            serde_json::from_slice(&body).expect("Failed to deserialize response");
        assert_eq!(response["message"], "Delete drink - TODO: implement");

        teardown_test_context(ctx).await;
    }
}
