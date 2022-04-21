use actix_web::{Error, dev::{self, Service, ServiceRequest, ServiceResponse, Transform}, HttpResponse};
use std::future::{ready, Ready};
use actix_web::body::EitherBody;
use futures_util::future::LocalBoxFuture;
use crate::model::api_key::ApiKeyDb;
use crate::web::DbPool;

static API_KEY_HEADER : &str = "x-api-key";

pub struct AuthMiddleware {
    pub pool: DbPool
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = DefaultAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(DefaultAuthMiddleware { service, pool: self.pool.clone() }))
    }
}

pub struct DefaultAuthMiddleware<S> {
    service: S,
    pool: DbPool
}

impl<S, B> Service<ServiceRequest> for DefaultAuthMiddleware<S>
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        if request.headers().contains_key(API_KEY_HEADER) {
            use crate::model::api_key::db::api_keys::*;
            use crate::model::api_key::db::api_keys::dsl::api_keys;
            use diesel::{RunQueryDsl, QueryDsl, ExpressionMethods};

            let conn = self.pool.get().map_err(actix_web::error::ErrorInternalServerError).unwrap();
            let keys : Vec<ApiKeyDb> = api_keys
                .filter(key.eq(request.headers().get(API_KEY_HEADER).unwrap().to_str().unwrap()))
                .limit(1)
                .load::<ApiKeyDb>(&conn)
                .expect("Unable to find API key entry");

            if keys.len() == 0 {
                let resp = HttpResponse::Unauthorized().finish().map_into_right_body();
                let (request, _pl) = request.into_parts();
                return Box::pin(async {
                    Ok(ServiceResponse::new(request, resp))
                });
            }
        } else {
            let resp = HttpResponse::Unauthorized().finish().map_into_right_body();
            let (request, _pl) = request.into_parts();
            return Box::pin(async {
                Ok(ServiceResponse::new(request, resp))
            });
        }

        let res = self.service.call(request);
        return Box::pin(async {
            res.await.map(ServiceResponse::map_into_left_body)
        });
    }
}