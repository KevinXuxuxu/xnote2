#[cfg(test)]
mod tests {
    use actix_web::{test, web, App};
    use serial_test::serial;
    use sqlx::PgPool;
    use xnote::handlers::activities;

    struct TestContext {
        pool: PgPool,
        activity1_id: i32,
        activity2_id: i32,
        _activity3_id: i32,
    }

    async fn create_test_database_pool() -> PgPool {
        dotenv::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        let last_slash = database_url.rfind('/').expect("DATABASE_URL must be a valid connection string");
        let database_url = format!("{}/xnote_test", &database_url[..last_slash]);
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
        // Drop leftover state from previous runs; init.sql inserts are not idempotent
        cleanup_database(&pool).await;
        create_schema(&pool).await;

        // Insert activities
        let activity1_id: i32 = sqlx::query_scalar!(
            "INSERT INTO activity (name, type) VALUES ($1, $2) RETURNING id",
            "Running",
            "sport"
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert activity 1");

        let activity2_id: i32 = sqlx::query_scalar!(
            "INSERT INTO activity (name, type) VALUES ($1, $2) RETURNING id",
            "Coding",
            "side project"
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert activity 2");

        let activity3_id: i32 = sqlx::query_scalar!(
            "INSERT INTO activity (name, type) VALUES ($1, $2) RETURNING id",
            "Cleaning",
            "chore"
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert activity 3");

        TestContext {
            pool,
            activity1_id,
            activity2_id,
            _activity3_id: activity3_id,
        }
    }

    async fn teardown_test_context(ctx: TestContext) {
        cleanup_database(&ctx.pool).await;
        ctx.pool.close().await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_activities_list() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(activities::configure),
        )
        .await;

        let req = test::TestRequest::get().uri("/activities").to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let activities: Vec<xnote::models::activity::Activity> =
            serde_json::from_slice(&body).expect("Failed to deserialize activities list");

        assert_eq!(activities.len(), 3);

        // Check activities are ordered by id (insertion order)
        assert_eq!(activities[0].name, "Running");
        assert_eq!(activities[1].name, "Coding");
        assert_eq!(activities[2].name, "Cleaning");

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_activity_by_id_running() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(activities::configure),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/activities/{}", ctx.activity1_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let activity: xnote::models::activity::Activity =
            serde_json::from_slice(&body).expect("Failed to deserialize activity");

        assert_eq!(activity.id, ctx.activity1_id);
        assert_eq!(activity.name, "Running");
        assert_eq!(activity.activity_type, "sport");

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_activity_by_id_coding() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(activities::configure),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/activities/{}", ctx.activity2_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let activity: xnote::models::activity::Activity =
            serde_json::from_slice(&body).expect("Failed to deserialize activity");

        assert_eq!(activity.id, ctx.activity2_id);
        assert_eq!(activity.name, "Coding");
        assert_eq!(activity.activity_type, "side project");

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_activity_by_id_not_found() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(activities::configure),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/activities/99999")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        let body = test::read_body(resp).await;
        let error_response: serde_json::Value =
            serde_json::from_slice(&body).expect("Failed to deserialize error response");
        assert_eq!(error_response["error"], "Activity not found");

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_create_activity() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(activities::configure),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/activities")
            .set_json(&serde_json::json!({
                "name": "Swimming",
                "type": "sport"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        let body = test::read_body(resp).await;
        let activity: xnote::models::activity::Activity =
            serde_json::from_slice(&body).expect("Failed to deserialize activity");
        assert_eq!(activity.name, "Swimming");
        assert_eq!(activity.activity_type, "sport");

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_update_activity() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(activities::configure),
        )
        .await;

        let req = test::TestRequest::put()
            .uri(&format!("/activities/{}", ctx.activity1_id))
            .set_json(&serde_json::json!({
                "name": "Running Fast"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body = test::read_body(resp).await;
        let activity: xnote::models::activity::Activity =
            serde_json::from_slice(&body).expect("Failed to deserialize activity");
        assert_eq!(activity.id, ctx.activity1_id);
        assert_eq!(activity.name, "Running Fast");
        assert_eq!(activity.activity_type, "sport");

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_delete_activity() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(activities::configure),
        )
        .await;

        let req = test::TestRequest::delete()
            .uri(&format!("/activities/{}", ctx.activity1_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body = test::read_body(resp).await;
        let response: serde_json::Value =
            serde_json::from_slice(&body).expect("Failed to deserialize response");
        assert_eq!(response["message"], "Activity deleted successfully");

        teardown_test_context(ctx).await;
    }
}
