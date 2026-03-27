use crate::{models::ApiResponse, AppState};
use actix_web::{web, HttpResponse};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    pub db: &'static str,
}

pub async fn health_check(state: web::Data<AppState>) -> HttpResponse {
    let db_status = match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => "ok",
        Err(_) => "error",
    };

    let status = if db_status == "ok" { "ok" } else { "degraded" };

    HttpResponse::Ok().json(ApiResponse::success(HealthResponse {
        status,
        version: env!("CARGO_PKG_VERSION"),
        db: db_status,
    }))
}
