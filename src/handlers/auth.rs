use crate::{
    db,
    models::{ApiResponse, AuthResponse, LoginRequest, RegisterRequest},
    utils::jwt::generate_token,
    AppState,
};
use actix_web::{web, HttpResponse};
use bcrypt::{hash, verify, DEFAULT_COST};
use tracing::{info, warn};

pub async fn register(
    state: web::Data<AppState>,
    body: web::Json<RegisterRequest>,
) -> HttpResponse {
    // Check duplicate email
    match db::find_user_by_email(&state.db, &body.email).await {
        Ok(Some(_)) => {
            return HttpResponse::Conflict()
                .json(ApiResponse::<()>::error("Email already registered"));
        }
        Err(e) => {
            warn!("DB error during register: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Internal server error"));
        }
        _ => {}
    }

    // Hash password
    let password_hash = match hash(&body.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to hash password"));
        }
    };

    // Create user
    match db::create_user(&state.db, &body.email, &body.username, &password_hash).await {
        Ok(user) => {
            info!("New user registered: {}", user.email);
            let token = generate_token(&user.id.to_string(), &user.email, &state.jwt_secret)
                .expect("Token generation failed");

            HttpResponse::Created().json(ApiResponse::success(AuthResponse {
                token,
                user: user.into(),
            }))
        }
        Err(e) => {
            warn!("Failed to create user: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to create user"))
        }
    }
}

pub async fn login(
    state: web::Data<AppState>,
    body: web::Json<LoginRequest>,
) -> HttpResponse {
    let user = match db::find_user_by_email(&state.db, &body.email).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return HttpResponse::Unauthorized()
                .json(ApiResponse::<()>::error("Invalid email or password"));
        }
        Err(e) => {
            warn!("DB error during login: {}", e);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Internal server error"));
        }
    };

    match verify(&body.password, &user.password_hash) {
        Ok(true) => {
            info!("User logged in: {}", user.email);
            let token = generate_token(&user.id.to_string(), &user.email, &state.jwt_secret)
                .expect("Token generation failed");

            HttpResponse::Ok().json(ApiResponse::success(AuthResponse {
                token,
                user: user.into(),
            }))
        }
        _ => HttpResponse::Unauthorized()
            .json(ApiResponse::<()>::error("Invalid email or password")),
    }
}
