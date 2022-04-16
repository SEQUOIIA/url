use actix_web::{middleware, web, App, HttpRequest, HttpServer, Error, dev::{self, Service, ServiceRequest, ServiceResponse, Transform}, Scope, Responder, HttpResponse};
use http::{HeaderValue, header::HeaderName};

use std::future::{ready, Ready};
use diesel::{RunQueryDsl, QueryDsl, ExpressionMethods, BelongingToDsl, SqliteConnection};
use diesel::r2d2::{self, ConnectionManager};
use futures_util::future::LocalBoxFuture;
use crate::model::url::UrlDb;
use crate::schema;
use log::info;

type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[actix_web::main]
pub async fn start_server() {
    let manager = ConnectionManager::<SqliteConnection>::new(crate::model::db::DATABASE_URL);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            // enable logger
            .wrap(middleware::Logger::default())
            .wrap(DefaultHeaders)
            .service(web::resource("/new").to(new_url_handler))
            .service(web::resource("/{id}").to(url_handler))
    })
        .bind(("0.0.0.0", 8380)).unwrap()
        .run()
        .await;
}

async fn url_handler(req: HttpRequest, pool: web::Data<DbPool>,) -> Result<HttpResponse, Error> {
    info!("url_handler triggered");
    if req.method().as_str() != "GET" {
        return Ok(HttpResponse::MethodNotAllowed().finish())
    }

    let conn = pool.get().map_err(actix_web::error::ErrorInternalServerError)?;
    let urls : Vec<UrlDb> = schema::urls::dsl::urls
        .filter(schema::urls::id.eq(req.path().strip_prefix('/').unwrap()))
        .limit(1)
        .load::<UrlDb>(&conn)
        .expect("Unable to find URL entry");

    if urls.len() > 0 {
        let url_entry = urls.first().unwrap();
        Ok(HttpResponse::TemporaryRedirect().insert_header(("Location", url_entry.url.as_str())).finish())
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

async fn new_url_handler(req: HttpRequest, body : web::Bytes) -> impl Responder {
    info!("new_url_handler triggered");
    if req.method().as_str() != "POST" {
        return HttpResponse::MethodNotAllowed().finish()
    }

    HttpResponse::Ok().finish()
}

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct DefaultHeaders;

// Middleware factory is `Transform` trait from actix-service crate
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for DefaultHeaders
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = DefaultHeadersMiddleware<S>;
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