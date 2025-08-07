#[cfg(test)]
mod tests {
    use actix_web::{test, web, App};
    use sqlx::PgPool;
    use xnote::handlers::meals;
    use xnote::models::detail::{MealDetail, MealFoodSource};
    use serial_test::serial;
    
    struct TestContext {
        pool: PgPool,
        meal1_id: i32,    // Restaurant meal with people
        meal2_id: i32,    // Recipe meal with one person  
        meal3_id: i32,    // Product meal with no people
        restaurant_id: i32,
        recipe_id: i32,
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
        
        // Insert restaurant
        let restaurant_id: i32 = sqlx::query_scalar!(
            "INSERT INTO restaurant (name, location, type, price) VALUES ($1, $2, $3, $4) RETURNING id",
            "Test Restaurant", "Seattle Downtown", "Italian", Some(25.50)
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert restaurant");
        
        // Insert recipe
        let recipe_id: i32 = sqlx::query_scalar!(
            "INSERT INTO recipe (name, ingredients, procedure, cautions) VALUES ($1, $2, $3, $4) RETURNING id",
            "Test Recipe", "flour, eggs, milk", "mix and bake", Some("hot oven")
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert recipe");
        
        // Insert product
        let product_id: i32 = sqlx::query_scalar!(
            "INSERT INTO product (name) VALUES ($1) RETURNING id",
            "Test Product"
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert product");
        
        // Insert meals
        let meal1_id: i32 = sqlx::query_scalar!(
            "INSERT INTO meal (date, \"time\", notes) VALUES ($1, $2, $3) RETURNING id",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "dinner",
            Some("Great dinner")
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert meal 1");
        
        let meal2_id: i32 = sqlx::query_scalar!(
            "INSERT INTO meal (date, \"time\", notes) VALUES ($1, $2, $3) RETURNING id",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
            "lunch", 
            None::<String>
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert meal 2");
        
        let meal3_id: i32 = sqlx::query_scalar!(
            "INSERT INTO meal (date, \"time\", notes) VALUES ($1, $2, $3) RETURNING id",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 17).unwrap(),
            "breakfast",
            Some("Quick breakfast")
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert meal 3");
        
        // Link meals to food sources
        sqlx::query!(
            "INSERT INTO meal_restaurant (meal, restaurant, type) VALUES ($1, $2, $3)",
            meal1_id, restaurant_id, "dine-in"
        )
        .execute(&pool)
        .await
        .expect("Failed to link meal 1 to restaurant");
        
        sqlx::query!(
            "INSERT INTO meal_recipe (meal, recipe, type) VALUES ($1, $2, $3)",
            meal2_id, recipe_id, "cooked"
        )
        .execute(&pool)
        .await
        .expect("Failed to link meal 2 to recipe");
        
        sqlx::query!(
            "INSERT INTO meal_product (meal, product, type) VALUES ($1, $2, $3)",
            meal3_id, product_id, "manufactured"
        )
        .execute(&pool)
        .await
        .expect("Failed to link meal 3 to product");
        
        // Link people to meals
        sqlx::query!(
            "INSERT INTO meal_people (meal, people) VALUES ($1, $2)",
            meal1_id, person1_id
        )
        .execute(&pool)
        .await
        .expect("Failed to link person 1 to meal 1");
        
        sqlx::query!(
            "INSERT INTO meal_people (meal, people) VALUES ($1, $2)",
            meal1_id, person2_id
        )
        .execute(&pool)
        .await
        .expect("Failed to link person 2 to meal 1");
        
        sqlx::query!(
            "INSERT INTO meal_people (meal, people) VALUES ($1, $2)",
            meal2_id, person1_id
        )
        .execute(&pool)
        .await
        .expect("Failed to link person 1 to meal 2");
        
        TestContext {
            pool,
            meal1_id,
            meal2_id,
            meal3_id,
            restaurant_id,
            recipe_id,
        }
    }
    
    async fn teardown_test_context(ctx: TestContext) {
        cleanup_database(&ctx.pool).await;
        ctx.pool.close().await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_meal_details_restaurant_with_people() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(meals::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri(&format!("/meals/{}/details", ctx.meal1_id))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let meal_detail: MealDetail = serde_json::from_slice(&body)
            .expect("Failed to deserialize meal detail");
            
        assert_eq!(meal_detail.id, ctx.meal1_id);
        assert_eq!(meal_detail.time, "dinner");
        assert_eq!(meal_detail.notes, Some("Great dinner".to_string()));
        assert_eq!(meal_detail.people.len(), 2);
        
        match meal_detail.food_source {
            Some(MealFoodSource::Restaurant { restaurant, meal_type }) => {
                assert_eq!(restaurant.id, ctx.restaurant_id);
                assert_eq!(restaurant.name, "Test Restaurant");
                assert_eq!(restaurant.location, "Seattle Downtown");
                assert_eq!(restaurant.food_type, "Italian");
                assert_eq!(restaurant.price, Some(25.50));
                assert_eq!(meal_type, "dine-in");
            }
            _ => panic!("Expected restaurant food source")
        }
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_meal_details_recipe_with_one_person() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(meals::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri(&format!("/meals/{}/details", ctx.meal2_id))
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let meal_detail: MealDetail = serde_json::from_slice(&body)
            .expect("Failed to deserialize meal detail");
            
        assert_eq!(meal_detail.id, ctx.meal2_id);
        assert_eq!(meal_detail.time, "lunch");
        assert_eq!(meal_detail.notes, None);
        assert_eq!(meal_detail.people.len(), 1);
        assert_eq!(meal_detail.people[0].name, "Alice");
        
        match meal_detail.food_source {
            Some(MealFoodSource::Recipe { recipe, meal_type }) => {
                assert_eq!(recipe.id, ctx.recipe_id);
                assert_eq!(recipe.name, "Test Recipe");
                assert_eq!(recipe.ingredients, "flour, eggs, milk");
                assert_eq!(recipe.procedure, "mix and bake");  
                assert_eq!(recipe.cautions, Some("hot oven".to_string()));
                assert_eq!(meal_type, "cooked");
            }
            _ => panic!("Expected recipe food source")
        }
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_meal_details_product_no_people() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(meals::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri(&format!("/meals/{}/details", ctx.meal3_id))
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let meal_detail: MealDetail = serde_json::from_slice(&body)
            .expect("Failed to deserialize meal detail");
            
        assert_eq!(meal_detail.id, ctx.meal3_id);
        assert_eq!(meal_detail.time, "breakfast");
        assert_eq!(meal_detail.notes, Some("Quick breakfast".to_string()));
        assert_eq!(meal_detail.people.len(), 0);
        
        match meal_detail.food_source {
            Some(MealFoodSource::Product { product, meal_type }) => {
                assert_eq!(product.name, "Test Product");
                assert_eq!(meal_type, "manufactured");
            }
            _ => panic!("Expected product food source")
        }
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_meal_details_not_found() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(meals::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri("/meals/99999/details")
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
        
        let body = test::read_body(resp).await;
        let error_response: serde_json::Value = serde_json::from_slice(&body)
            .expect("Failed to deserialize error response");
        assert_eq!(error_response["error"], "Meal not found");
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_get_meals_list() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(meals::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri("/meals")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let meals: Vec<xnote::models::meal::Meal> = serde_json::from_slice(&body)
            .expect("Failed to deserialize meals list");
            
        assert_eq!(meals.len(), 3);
        
        // Check meals are ordered by date DESC, time
        assert_eq!(meals[0].time, "breakfast"); // 2024-01-17
        assert_eq!(meals[1].time, "lunch");     // 2024-01-16
        assert_eq!(meals[2].time, "dinner");   // 2024-01-15
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_get_meal_by_id_restaurant() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(meals::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri(&format!("/meals/{}", ctx.meal1_id))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let meal: xnote::models::meal::Meal = serde_json::from_slice(&body)
            .expect("Failed to deserialize meal");
            
        assert_eq!(meal.id, ctx.meal1_id);
        assert_eq!(meal.time, "dinner");
        assert_eq!(meal.notes, Some("Great dinner".to_string()));
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_get_meal_by_id_recipe() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(meals::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri(&format!("/meals/{}", ctx.meal2_id))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let meal: xnote::models::meal::Meal = serde_json::from_slice(&body)
            .expect("Failed to deserialize meal");
            
        assert_eq!(meal.id, ctx.meal2_id);
        assert_eq!(meal.time, "lunch");
        assert_eq!(meal.notes, None);
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_get_meal_by_id_product() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(meals::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri(&format!("/meals/{}", ctx.meal3_id))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let meal: xnote::models::meal::Meal = serde_json::from_slice(&body)
            .expect("Failed to deserialize meal");
            
        assert_eq!(meal.id, ctx.meal3_id);
        assert_eq!(meal.time, "breakfast");
        assert_eq!(meal.notes, Some("Quick breakfast".to_string()));
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_get_meal_by_id_not_found() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(meals::configure)
        ).await;
        
        let req = test::TestRequest::get()
            .uri("/meals/99999")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
        
        let body = test::read_body(resp).await;
        let error_response: serde_json::Value = serde_json::from_slice(&body)
            .expect("Failed to deserialize error response");
        assert_eq!(error_response["error"], "Meal not found");
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_create_meal_placeholder() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(meals::configure)
        ).await;
        
        let req = test::TestRequest::post()
            .uri("/meals")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
        
        let body = test::read_body(resp).await;
        let response: serde_json::Value = serde_json::from_slice(&body)
            .expect("Failed to deserialize response");
        assert_eq!(response["message"], "Create meal - TODO: implement");
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_update_meal_placeholder() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(meals::configure)
        ).await;
        
        let req = test::TestRequest::put()
            .uri(&format!("/meals/{}", ctx.meal1_id))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let response: serde_json::Value = serde_json::from_slice(&body)
            .expect("Failed to deserialize response");
        assert_eq!(response["message"], "Update meal - TODO: implement");
        
        teardown_test_context(ctx).await;
    }
    
    #[actix_web::test]
    #[serial]
    async fn test_delete_meal_placeholder() {
        let ctx = setup_test_context().await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ctx.pool.clone()))
                .configure(meals::configure)
        ).await;
        
        let req = test::TestRequest::delete()
            .uri(&format!("/meals/{}", ctx.meal1_id))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let response: serde_json::Value = serde_json::from_slice(&body)
            .expect("Failed to deserialize response");
        assert_eq!(response["message"], "Delete meal - TODO: implement");
        
        teardown_test_context(ctx).await;
    }
}