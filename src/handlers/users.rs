use crate::{
    db,
    models::{ApiResponse, Claims, UpdateUserRequest, UserResponse},
    AppState,
};
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use uuid::Uuid;

pub async fn get_me(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let claims = req.extensions().get::<Claims>().cloned().unwrap();
    let id = Uuid::parse_str(&claims.sub).unwrap();

    match db::find_user_by_id(&state.db, id).await {
        Ok(Some(user)) => HttpResponse::Ok().json(ApiResponse::success(UserResponse::from(user))),
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()>::error("User not found")),
        Err(_) => HttpResponse::InternalServerError()
            .json(ApiResponse::<()>::error("Internal server error")),
    }
}

pub async fn list_users(
    state: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let limit = query.get("limit").and_then(|v| v.parse().ok()).unwrap_or(20i64);
    let offset = query.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0i64);

    match db::list_users(&state.db, limit, offset).await {
        Ok(users) => {
            let response: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
            HttpResponse::Ok().json(ApiResponse::success(response))
        }
        Err(_) => HttpResponse::InternalServerError()
            .json(ApiResponse::<()>::error("Internal server error")),
    }
}

pub async fn get_user(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let id = path.into_inner();

    match db::find_user_by_id(&state.db, id).await {
        Ok(Some(user)) => HttpResponse::Ok().json(ApiResponse::success(UserResponse::from(user))),
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()>::error("User not found")),
        Err(_) => HttpResponse::InternalServerError()
            .json(ApiResponse::<()>::error("Internal server error")),
    }
}

pub async fn update_user(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateUserRequest>,
) -> HttpResponse {
    let id = path.into_inner();

    match db::update_user(
        &state.db,
        id,
        body.username.as_deref(),
        body.email.as_deref(),
    )
    .await
    {
        Ok(Some(user)) => HttpResponse::Ok().json(ApiResponse::success(UserResponse::from(user))),
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()>::error("User not found")),
        Err(_) => HttpResponse::InternalServerError()
            .json(ApiResponse::<()>::error("Internal server error")),
    }
}

pub async fn delete_user(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let id = path.into_inner();

    match db::delete_user(&state.db, id).await {
        Ok(true) => HttpResponse::Ok().json(ApiResponse::<()>::message("User deleted")),
        Ok(false) => HttpResponse::NotFound().json(ApiResponse::<()>::error("User not found")),
        Err(_) => HttpResponse::InternalServerError()
            .json(ApiResponse::<()>::error("Internal server error")),
    }
}
