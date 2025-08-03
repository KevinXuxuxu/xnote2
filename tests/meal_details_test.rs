#[cfg(test)]
mod tests {
    use actix_web::{test, web, App};
    use sqlx::{PgPool, Row};
    use xnote::handlers::meals;
    use xnote::models::detail::{MealDetail, MealFoodSource};
    
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
    
    async fn setup_test_data(pool: &PgPool) -> (i32, i32, i32, i32, i32) {
        // Insert people
        let person1_id: i32 = sqlx::query_scalar!(
            "INSERT INTO people (name, notes) VALUES ($1, $2) RETURNING id",
            "Alice", Some("Test person 1")
        )
        .fetch_one(pool)
        .await
        .expect("Failed to insert person 1");
        
        let person2_id: i32 = sqlx::query_scalar!(
            "INSERT INTO people (name, notes) VALUES ($1, $2) RETURNING id", 
            "Bob", None::<String>
        )
        .fetch_one(pool)
        .await
        .expect("Failed to insert person 2");
        
        // Insert restaurant
        let restaurant_id: i32 = sqlx::query_scalar!(
            "INSERT INTO restaurant (name, location, type, price) VALUES ($1, $2, $3, $4) RETURNING id",
            "Test Restaurant", "Seattle Downtown", "Italian", Some(25.50)
        )
        .fetch_one(pool)
        .await
        .expect("Failed to insert restaurant");
        
        // Insert recipe
        let recipe_id: i32 = sqlx::query_scalar!(
            "INSERT INTO recipe (name, ingredients, procedure, cautions) VALUES ($1, $2, $3, $4) RETURNING id",
            "Test Recipe", "flour, eggs, milk", "mix and bake", Some("hot oven")
        )
        .fetch_one(pool)
        .await
        .expect("Failed to insert recipe");
        
        // Insert product
        let product_id: i32 = sqlx::query_scalar!(
            "INSERT INTO product (name) VALUES ($1) RETURNING id",
            "Test Product"
        )
        .fetch_one(pool)
        .await
        .expect("Failed to insert product");
        
        // Insert meals with restaurant
        let meal1_id: i32 = sqlx::query_scalar!(
            "INSERT INTO meal (date, \"time\", notes) VALUES ($1, $2, $3) RETURNING id",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            "dinner",
            Some("Great dinner")
        )
        .fetch_one(pool)
        .await
        .expect("Failed to insert meal 1");
        
        // Insert meal with recipe
        let meal2_id: i32 = sqlx::query_scalar!(
            "INSERT INTO meal (date, \"time\", notes) VALUES ($1, $2, $3) RETURNING id",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
            "lunch", 
            None::<String>
        )
        .fetch_one(pool)
        .await
        .expect("Failed to insert meal 2");
        
        // Insert meal with product (no people)
        let meal3_id: i32 = sqlx::query_scalar!(
            "INSERT INTO meal (date, \"time\", notes) VALUES ($1, $2, $3) RETURNING id",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 17).unwrap(),
            "breakfast",
            Some("Quick breakfast")
        )
        .fetch_one(pool)
        .await
        .expect("Failed to insert meal 3");
        
        // Link meal 1 to restaurant
        sqlx::query!(
            "INSERT INTO meal_restaurant (meal, restaurant, type) VALUES ($1, $2, $3)",
            meal1_id, restaurant_id, "dine-in"
        )
        .execute(pool)
        .await
        .expect("Failed to link meal 1 to restaurant");
        
        // Link meal 2 to recipe
        sqlx::query!(
            "INSERT INTO meal_recipe (meal, recipe, type) VALUES ($1, $2, $3)",
            meal2_id, recipe_id, "cooked"
        )
        .execute(pool)
        .await
        .expect("Failed to link meal 2 to recipe");
        
        // Link meal 3 to product
        sqlx::query!(
            "INSERT INTO meal_product (meal, product, type) VALUES ($1, $2, $3)",
            meal3_id, product_id, "manufactured"
        )
        .execute(pool)
        .await
        .expect("Failed to link meal 3 to product");
        
        // Link people to meals
        sqlx::query!(
            "INSERT INTO meal_people (meal, people) VALUES ($1, $2)",
            meal1_id, person1_id
        )
        .execute(pool)
        .await
        .expect("Failed to link person 1 to meal 1");
        
        sqlx::query!(
            "INSERT INTO meal_people (meal, people) VALUES ($1, $2)",
            meal1_id, person2_id
        )
        .execute(pool)
        .await
        .expect("Failed to link person 2 to meal 1");
        
        sqlx::query!(
            "INSERT INTO meal_people (meal, people) VALUES ($1, $2)",
            meal2_id, person1_id
        )
        .execute(pool)
        .await
        .expect("Failed to link person 1 to meal 2");
        
        (meal1_id, meal2_id, meal3_id, restaurant_id, recipe_id)
    }
    
    #[actix_web::test]
    async fn test_meal_details_comprehensive() {
        // Setup database
        let pool = create_test_database_pool().await;
        create_schema(&pool).await;
        
        let (meal1_id, meal2_id, meal3_id, restaurant_id, recipe_id) = setup_test_data(&pool).await;
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .configure(meals::configure)
        ).await;
        
        // Test 1: Restaurant meal with people
        let req = test::TestRequest::get()
            .uri(&format!("/meals/{}/details", meal1_id))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let meal_detail: MealDetail = serde_json::from_slice(&body)
            .expect("Failed to deserialize meal detail");
            
        assert_eq!(meal_detail.id, meal1_id);
        assert_eq!(meal_detail.time, "dinner");
        assert_eq!(meal_detail.notes, Some("Great dinner".to_string()));
        assert_eq!(meal_detail.people.len(), 2);
        
        match meal_detail.food_source {
            Some(MealFoodSource::Restaurant { restaurant, meal_type }) => {
                assert_eq!(restaurant.id, restaurant_id);
                assert_eq!(restaurant.name, "Test Restaurant");
                assert_eq!(restaurant.location, "Seattle Downtown");
                assert_eq!(restaurant.food_type, "Italian");
                assert_eq!(restaurant.price, Some(25.50));
                assert_eq!(meal_type, "dine-in");
            }
            _ => panic!("Expected restaurant food source")
        }
        
        // Test 2: Recipe meal with one person
        let req = test::TestRequest::get()
            .uri(&format!("/meals/{}/details", meal2_id))
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let meal_detail: MealDetail = serde_json::from_slice(&body)
            .expect("Failed to deserialize meal detail");
            
        assert_eq!(meal_detail.id, meal2_id);
        assert_eq!(meal_detail.time, "lunch");
        assert_eq!(meal_detail.notes, None);
        assert_eq!(meal_detail.people.len(), 1);
        assert_eq!(meal_detail.people[0].name, "Alice");
        
        match meal_detail.food_source {
            Some(MealFoodSource::Recipe { recipe, meal_type }) => {
                assert_eq!(recipe.id, recipe_id);
                assert_eq!(recipe.name, "Test Recipe");
                assert_eq!(recipe.ingredients, "flour, eggs, milk");
                assert_eq!(recipe.procedure, "mix and bake");
                assert_eq!(recipe.cautions, Some("hot oven".to_string()));
                assert_eq!(meal_type, "cooked");
            }
            _ => panic!("Expected recipe food source")
        }
        
        // Test 3: Product meal with no people
        let req = test::TestRequest::get()
            .uri(&format!("/meals/{}/details", meal3_id))
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let body = test::read_body(resp).await;
        let meal_detail: MealDetail = serde_json::from_slice(&body)
            .expect("Failed to deserialize meal detail");
            
        assert_eq!(meal_detail.id, meal3_id);
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
        
        // Test 4: Non-existent meal
        let req = test::TestRequest::get()
            .uri("/meals/99999/details")
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
        
        let body = test::read_body(resp).await;
        let error_response: serde_json::Value = serde_json::from_slice(&body)
            .expect("Failed to deserialize error response");
        assert_eq!(error_response["error"], "Meal not found");
        
        // Cleanup
        cleanup_database(&pool).await;
        pool.close().await;
    }
}