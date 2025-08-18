use actix_web::{web, App, HttpResponse, HttpServer, Result, middleware::Logger};
use actix_files as fs;
use sqlx::PgPool;
use std::env;

mod models;
mod config;
mod handlers;

async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "daily-events-service"
    })))
}

async fn index() -> Result<HttpResponse> {
    let html = std::fs::read_to_string("static/index.html")
        .unwrap_or_else(|_| {
            r#"<!DOCTYPE html>
<html>
<head>
    <title>XNote - Daily Events Tracker</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
</head>
<body>
    <h1>XNote - Daily Events Tracker</h1>
    <p>Frontend is being set up...</p>
</body>
</html>"#.to_string()
        });
    
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    log::info!("Starting server on http://0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(Logger::default())
            .route("/", web::get().to(index))
            .route("/health", web::get().to(health))
            .service(fs::Files::new("/static", "./static").show_files_listing())
            .service(
                web::scope("/api/v1")
                    .configure(handlers::meals::configure)
                    .configure(handlers::events::configure)
                    .configure(handlers::people::configure)
                    .configure(handlers::locations::configure)
                    .configure(handlers::restaurants::configure)
                    .configure(handlers::drinks::configure)
                    .configure(handlers::drink_options::configure)
                    .configure(handlers::recipes::configure)
                    .configure(handlers::products::configure)
                    .configure(handlers::activities::configure)
                    .configure(handlers::daily_summary::configure)
                    .configure(handlers::food_types::configure)
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}