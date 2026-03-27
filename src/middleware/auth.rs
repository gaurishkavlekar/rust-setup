use crate::{models::ApiResponse, utils::jwt::verify_token, AppState};
use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, HttpMessage, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
};

pub struct JwtMiddleware;

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct JwtMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for JwtMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            // Extract Bearer token from Authorization header
            let token = req
                .headers()
                .get("Authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .map(|s| s.to_string());

            let token = match token {
                Some(t) => t,
                None => {
                    let response = HttpResponse::Unauthorized()
                        .json(ApiResponse::<()>::error("Missing Authorization header"))
                        .map_into_right_body();
                    return Ok(req.into_response(response));
                }
            };

            // Verify token against app state secret
            let state = req.app_data::<web::Data<AppState>>().cloned();
            let claims = match state {
                Some(s) => verify_token(&token, &s.jwt_secret),
                None => {
                    let response = HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error("Server configuration error"))
                        .map_into_right_body();
                    return Ok(req.into_response(response));
                }
            };

            match claims {
                Ok(c) => {
                    // Attach claims to request extensions for handlers to use
                    req.extensions_mut().insert(c);
                    let res = svc.call(req).await?.map_into_left_body();
                    Ok(res)
                }
                Err(_) => {
                    let response = HttpResponse::Unauthorized()
                        .json(ApiResponse::<()>::error("Invalid or expired token"))
                        .map_into_right_body();
                    Ok(req.into_response(response))
                }
            }
        })
    }
}