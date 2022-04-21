use actix_web::{middleware, web, App, HttpRequest, HttpServer, Error, dev::{self, Service, ServiceRequest, ServiceResponse, Transform}, Scope, Responder, HttpResponse};
use http::{HeaderValue, header::HeaderName};

use std::future::{ready, Ready};
use actix_web::web::Bytes;
use diesel::{RunQueryDsl, QueryDsl, ExpressionMethods, BelongingToDsl, SqliteConnection};
use diesel::r2d2::{self, ConnectionManager};
use futures_util::future::LocalBoxFuture;
use crate::model::url::{UrlDb, UrlDbInsert, UrlRequest};
use crate::schema;
use log::info;
use serde::Deserialize;
use crate::api::{DefaultHeaders, AuthMiddleware};
use crate::model::api_key::{ApiKeyDb, ApiKeyDbInsert, ApiKeyDeleteRequest, ApiKeyPostRequest, ApiKeyPostResponse};

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

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
            .service(web::resource("/new").to(new_url_handler)).wrap(AuthMiddleware {pool: pool.clone()})
            .service(web::resource("/key").to(key_handler)).wrap(AuthMiddleware {pool: pool.clone()})
            .service(web::resource("/{id}").to(url_handler))
    })
        .bind(("0.0.0.0", 8380)).unwrap()
        .run()
        .await;
}

async fn url_handler(req: HttpRequest, pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
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

nano_id::gen!(
    url_id,
    62,
    b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
);

async fn new_url_handler(req: HttpRequest, pool: web::Data<DbPool>, body : web::Bytes) -> Result<HttpResponse, Error> {
    if req.method().as_str() != "POST" {
        return Ok(HttpResponse::MethodNotAllowed().finish());
    }
    let conn = pool.get().map_err(actix_web::error::ErrorInternalServerError)?;
    let req_body : UrlRequest = serde_json::from_slice(&body).map_err(actix_web::error::ErrorBadRequest)?;
    let id = url_id::<5>();

    let db_entry = UrlDbInsert {
        id: id.clone(),
        url: req_body.url
    };
    diesel::insert_into(schema::urls::table)
        .values(&db_entry)
        .execute(&conn)
        .expect("Unable to create URL entry");

    Ok(HttpResponse::Ok().body(format!("http://localhost:8380/{}", id)))
}

nano_id::gen!(
    api_key,
    86,
    b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ&*/@!#$%^()-_=+[]{};:,.?"
);

async fn key_handler(req: HttpRequest, pool: web::Data<DbPool>, body : web::Bytes) -> Result<HttpResponse, Error> {
    use crate::model::api_key::db::api_keys::*;
    use crate::model::api_key::db::api_keys::dsl::api_keys;
    return match req.method().as_str() {
        "POST" => {
            let req_body : ApiKeyPostRequest = serde_json::from_slice(&body).map_err(actix_web::error::ErrorBadRequest)?;

            let new_key = api_key::<64>();
            let conn = pool.get().map_err(actix_web::error::ErrorInternalServerError)?;
            let db_entry = ApiKeyDbInsert {
                key: new_key.clone(),
                description: req_body.description
            };

            diesel::insert_into(crate::model::api_key::db::api_keys::table)
                .values(&db_entry)
                .execute(&conn)
                .expect("Unable to crate API key entry");

            let resp = ApiKeyPostResponse {
                key: new_key
            };

            Ok(HttpResponse::Ok().json(&resp))
        },
        "DELETE" => {
            let req_body : ApiKeyDeleteRequest = serde_json::from_slice(&body).map_err(actix_web::error::ErrorBadRequest)?;

            let conn = pool.get().map_err(actix_web::error::ErrorInternalServerError)?;
            let keys : Vec<ApiKeyDb> = api_keys
                .filter(key.eq(req_body.key))
                .limit(1)
                .load::<ApiKeyDb>(&conn)
                .expect("Unable to find API key entry");

            return if keys.len() > 0 {
                let api_key_entry = keys.first().unwrap();
                diesel::delete(api_keys.filter(key.eq(&api_key_entry.key))).execute(&conn);
                Ok(HttpResponse::Ok().finish())
            } else {
                Ok(HttpResponse::NotFound().finish())
            }
        },
        _ => Ok(HttpResponse::MethodNotAllowed().finish())
    }
}