#[cfg(test)]
mod tests {
    use actix_web::{test, web, App};
    use serial_test::serial;
    use sqlx::PgPool;
    use xnote::handlers::recipes;

    struct TestContext {
        pool: PgPool,
        recipe1_id: i32,
        recipe2_id: i32,
        _recipe3_id: i32,
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

        // Insert recipes
        let recipe1_id: i32 = sqlx::query_scalar!(
            "INSERT INTO recipe (name, ingredients, procedure, cautions) VALUES ($1, $2, $3, $4) RETURNING id",
            "Pancakes", "flour, eggs, milk", "mix and cook", Some("hot pan")
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert recipe 1");

        let recipe2_id: i32 = sqlx::query_scalar!(
            "INSERT INTO recipe (name, ingredients, procedure, cautions) VALUES ($1, $2, $3, $4) RETURNING id",
            "Pasta", "pasta, tomato sauce", "boil and mix", None::<String>
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert recipe 2");

        let recipe3_id: i32 = sqlx::query_scalar!(
            "INSERT INTO recipe (name, ingredients, procedure, cautions) VALUES ($1, $2, $3, $4) RETURNING id",
            "Soup", "vegetables, broth", "simmer for 30 minutes", Some("avoid overcooking")
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert recipe 3");

        TestContext {
            pool,
            recipe1_id,
            recipe2_id,
            _recipe3_id: recipe3_id,
        }
    }

    async fn teardown_test_context(ctx: TestContext) {
        cleanup_database(&ctx.pool).await;
        ctx.pool.close().await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_recipes_list() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(recipes::configure),
        )
        .await;

        let req = test::TestRequest::get().uri("/recipes").to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let recipes: Vec<xnote::models::recipe::Recipe> =
            serde_json::from_slice(&body).expect("Failed to deserialize recipes list");

        assert_eq!(recipes.len(), 3);

        // Check recipes are ordered by id (insertion order)
        assert_eq!(recipes[0].name, "Pancakes");
        assert_eq!(recipes[1].name, "Pasta");
        assert_eq!(recipes[2].name, "Soup");

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_recipe_by_id_pancakes() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(recipes::configure),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/recipes/{}", ctx.recipe1_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let recipe: xnote::models::recipe::Recipe =
            serde_json::from_slice(&body).expect("Failed to deserialize recipe");

        assert_eq!(recipe.id, ctx.recipe1_id);
        assert_eq!(recipe.name, "Pancakes");
        assert_eq!(recipe.ingredients, "flour, eggs, milk");
        assert_eq!(recipe.procedure, "mix and cook");
        assert_eq!(recipe.cautions, Some("hot pan".to_string()));

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_recipe_by_id_pasta() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(recipes::configure),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/recipes/{}", ctx.recipe2_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let recipe: xnote::models::recipe::Recipe =
            serde_json::from_slice(&body).expect("Failed to deserialize recipe");

        assert_eq!(recipe.id, ctx.recipe2_id);
        assert_eq!(recipe.name, "Pasta");
        assert_eq!(recipe.ingredients, "pasta, tomato sauce");
        assert_eq!(recipe.procedure, "boil and mix");
        assert_eq!(recipe.cautions, None);

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_get_recipe_by_id_not_found() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(recipes::configure),
        )
        .await;

        let req = test::TestRequest::get().uri("/recipes/99999").to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        let body = test::read_body(resp).await;
        let error_response: serde_json::Value =
            serde_json::from_slice(&body).expect("Failed to deserialize error response");
        assert_eq!(error_response["error"], "Recipe not found");

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_create_recipe() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(recipes::configure),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/recipes")
            .set_json(&serde_json::json!({
                "name": "Salad",
                "ingredients": "lettuce, tomato",
                "procedure": "chop and toss",
                "cautions": null
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        let body = test::read_body(resp).await;
        let recipe: xnote::models::recipe::Recipe =
            serde_json::from_slice(&body).expect("Failed to deserialize recipe");
        assert_eq!(recipe.name, "Salad");
        assert_eq!(recipe.ingredients, "lettuce, tomato");
        assert_eq!(recipe.cautions, None);

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_update_recipe() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(recipes::configure),
        )
        .await;

        let req = test::TestRequest::put()
            .uri(&format!("/recipes/{}", ctx.recipe1_id))
            .set_json(&serde_json::json!({
                "name": "Pancakes Deluxe"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body = test::read_body(resp).await;
        let recipe: xnote::models::recipe::Recipe =
            serde_json::from_slice(&body).expect("Failed to deserialize recipe");
        assert_eq!(recipe.id, ctx.recipe1_id);
        assert_eq!(recipe.name, "Pancakes Deluxe");
        assert_eq!(recipe.ingredients, "flour, eggs, milk");

        teardown_test_context(ctx).await;
    }

    #[actix_web::test]
    #[serial]
    async fn test_delete_recipe() {
        let ctx = setup_test_context().await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(recipes::configure),
        )
        .await;

        let req = test::TestRequest::delete()
            .uri(&format!("/recipes/{}", ctx.recipe1_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body = test::read_body(resp).await;
        let response: serde_json::Value =
            serde_json::from_slice(&body).expect("Failed to deserialize response");
        assert_eq!(response["message"], "Recipe deleted successfully");

        teardown_test_context(ctx).await;
    }
}
