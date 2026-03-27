mod db;
mod handlers;
mod middleware;
mod models;
mod utils;

use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv::dotenv;
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use tracing::info;
use tracing_actix_web::TracingLogger;

pub struct AppState {
    pub db: sqlx::MySqlPool,
    pub jwt_secret: String,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    utils::logging::init_tracing();

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    info!("Connecting to database...");
    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    info!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;

    let app_state = web::Data::new(AppState {
        db: pool,
        jwt_secret,
    });

    let bind_addr = format!("{}:{}", host, port);
    info!("Starting server at http://{}", bind_addr);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(TracingLogger::default())
            .wrap(Logger::default())
            // Public routes
            .service(
                web::scope("/api/v1")
                    .service(
                        web::scope("/auth")
                            .route("/register", web::post().to(handlers::auth::register))
                            .route("/login", web::post().to(handlers::auth::login)),
                    )
                    // Protected routes
                    .service(
                        web::scope("/users")
                            .wrap(middleware::auth::JwtMiddleware)
                            .route("", web::get().to(handlers::users::list_users))
                            .route("/me", web::get().to(handlers::users::get_me))
                            .route("/{id}", web::get().to(handlers::users::get_user))
                            .route("/{id}", web::put().to(handlers::users::update_user))
                            .route("/{id}", web::delete().to(handlers::users::delete_user)),
                    )
                    .route("/health", web::get().to(handlers::health::health_check)),
            )
    })
    .bind(&bind_addr)?
    .run()
    .await?;

    Ok(())
}
