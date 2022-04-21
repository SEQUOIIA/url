use actix_web::{middleware, web, App, HttpRequest, HttpServer, Error, dev::{self, Service, ServiceRequest, ServiceResponse, Transform}, Scope, Responder, HttpResponse};
use std::future::{ready, Ready};
use http::{HeaderValue, header::HeaderName};
use futures_util::future::LocalBoxFuture;

pub struct DefaultHeaders;

impl<S, B> Transform<S, ServiceRequest> for DefaultHeaders
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = DefaultHeadersMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(DefaultHeadersMiddleware { service }))
    }
}

pub struct DefaultHeadersMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for DefaultHeadersMiddleware<S>
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            res.headers_mut().insert(HeaderName::from_static("server"), HeaderValue::from_static("url"));

            Ok(res)
        })
    }
}