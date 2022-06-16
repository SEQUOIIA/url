use std::str::FromStr;
use actix_web::{middleware, web, App, HttpRequest, HttpServer, Error, HttpResponse};
use diesel::{RunQueryDsl, QueryDsl, ExpressionMethods, SqliteConnection};
use diesel::r2d2::{self, ConnectionManager};
use diesel::result::DatabaseErrorKind;
use crate::model::url::{UrlDb, UrlDbInsert, UrlDeleteRequest, UrlRequest};
use crate::schema;
use log::info;
use thiserror::private::DisplayAsDisplay;
use crate::api::{DefaultHeaders, AuthMiddleware};
use crate::model::api_key::{ApiKey, ApiKeyDb, ApiKeyDbInsert, ApiKeyDeleteRequest, ApiKeyPostRequest, ApiKeyPostResponse};
use crate::model::db::{DATABASE_URL, get_db_path};
use crate::model::error::{url_err_any, url_err_request};
use crate::model::error::Error::RequestError;

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[actix_web::main]
pub async fn start_server() {
    let manager = ConnectionManager::<SqliteConnection>::new(format!("{}/{}", get_db_path(), DATABASE_URL));
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            // enable logger
            .wrap(middleware::Logger::default())
            .wrap(DefaultHeaders)
            .service(web::resource("/new").to(new_url_handler).wrap(AuthMiddleware {pool: pool.clone()}))
            .service(web::resource("/delete").to(delete_url_handler).wrap(AuthMiddleware {pool: pool.clone()}))
            .service(web::resource("/key").to(key_handler).wrap(AuthMiddleware {pool: pool.clone()}))
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
    let path = req.path().strip_prefix('/').unwrap().to_string();

    let urls : Vec<UrlDb> = web::block(move || {
        let conn = pool.get().map_err(url_err_any)?;
        schema::urls::dsl::urls
            .filter(schema::urls::id.eq(path))
            .limit(1)
            .load::<UrlDb>(&conn)
            .map_err(url_err_any)
    })
        .await?
        .map_err(actix_web::error::ErrorInternalServerError)?;

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
    let req_body : UrlRequest = serde_json::from_slice(&body).map_err(actix_web::error::ErrorBadRequest)?;
    // Make sure the URL given in the request body is valid
    url::Url::parse(&req_body.url).map_err(url_err_request)?;

    let id = match req_body.id {
        Some(val) => val,
        None => url_id::<5>()
    };

    let db_entry = UrlDbInsert {
        id: id.clone(),
        url: req_body.url
    };

    let db_resp = web::block( move || {
        let conn = pool.get().map_err(url_err_any)?;
        diesel::insert_into(schema::urls::table)
            .values(&db_entry)
            .execute(&conn)
            .map_err(url_err_any)
    }).await?;
    if let Err(val) = db_resp {
        if let crate::model::error::Error::Any(inner_err) = &val {
            let db_err : &diesel::result::Error = inner_err.downcast_ref().unwrap();
            return if let diesel::result::Error::DatabaseError(kind, _) = db_err {
                match kind {
                    DatabaseErrorKind::UniqueViolation => {
                        Err("URL name already in use. Try a different one".to_owned()).map_err(actix_web::error::ErrorBadRequest)
                    }
                    _ => {
                        println!("{}", db_err.to_string());
                        Err("Database error".to_owned()).map_err(actix_web::error::ErrorInternalServerError)
                    }
                }
            } else {
                println!("{}", val.err_msg());
                Err(val).map_err(actix_web::error::ErrorInternalServerError)
            }
        }
    }

    Ok(HttpResponse::Ok().body(format!("http://localhost:8380/{}", id)))
}

async fn delete_url_handler(req: HttpRequest, pool: web::Data<DbPool>, body : web::Bytes) -> Result<HttpResponse, Error> {
    use crate::schema::urls::*;
    use crate::schema::urls::dsl::urls;

    if req.method().as_str() != "DELETE" {
        return Ok(HttpResponse::MethodNotAllowed().finish());
    }
    let req_body : UrlDeleteRequest = serde_json::from_slice(&body).map_err(actix_web::error::ErrorBadRequest)?;

    let db_resp = web::block( move || {
        let conn = pool.get().map_err(url_err_any)?;
        diesel::delete(urls.filter(id.eq(req_body.id)))
            .execute(&conn)
            .map_err(url_err_any)
    }).await?;
    if let Err(val) = db_resp {
        if let crate::model::error::Error::Any(inner_err) = &val {
            let db_err : &diesel::result::Error = inner_err.downcast_ref().unwrap();
            return if let diesel::result::Error::DatabaseError(kind, _) = db_err {
                println!("{}", db_err.to_string());
                Err("Database error".to_owned()).map_err(actix_web::error::ErrorInternalServerError)
            } else {
                println!("{}", val.err_msg());
                Err(val).map_err(actix_web::error::ErrorInternalServerError)
            }
        }
    } else {
        if db_resp.unwrap() == 0 {
            return Ok(HttpResponse::NotFound().finish());
        }
    }

    Ok(HttpResponse::Ok().finish())
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
        "GET" => {

            let keys : Vec<ApiKeyDb> = web::block(move || {
                let conn = pool.get().map_err(url_err_any)?;
                api_keys
                    .load::<ApiKeyDb>(&conn)
                    .map_err(url_err_any)
            }).await?
                .map_err(actix_web::error::ErrorInternalServerError)?;

           let keys_resp : Vec<ApiKey> = keys.iter()
                .map(|a| {
                    ApiKey {
                        id: a.id,
                        key: a.key.clone(),
                        description: a.description.clone()
                    }
                }).collect();


            Ok(HttpResponse::Ok().json(&keys_resp))
        },
        "POST" => {
            let req_body : ApiKeyPostRequest = serde_json::from_slice(&body).map_err(actix_web::error::ErrorBadRequest)?;

            let new_key = api_key::<64>();
            let db_entry = ApiKeyDbInsert {
                key: new_key.clone(),
                description: req_body.description
            };

            web::block(move || {
                let conn = pool.get().map_err(url_err_any)?;
                diesel::insert_into(crate::model::api_key::db::api_keys::table)
                    .values(&db_entry)
                    .execute(&conn)
                    .map_err(url_err_any)
            }).await?
                .map_err(actix_web::error::ErrorInternalServerError)?;


            let resp = ApiKeyPostResponse {
                key: new_key
            };

            Ok(HttpResponse::Ok().json(&resp))
        },
        "DELETE" => {
            let req_body : ApiKeyDeleteRequest = serde_json::from_slice(&body).map_err(actix_web::error::ErrorBadRequest)?;

            let moved_pool = pool.clone();
            let keys : Vec<ApiKeyDb> = web::block(move || {
                let conn = moved_pool.get().map_err(url_err_any)?;
                api_keys
                    .filter(key.eq(req_body.key))
                    .limit(1)
                    .load::<ApiKeyDb>(&conn)
                    .map_err(url_err_any)
            }).await?
                .map_err(actix_web::error::ErrorInternalServerError)?;


            return if keys.len() > 0 {
                let api_key_entry = keys.first().unwrap().clone();
                web::block(move || {
                    let conn = pool.get().map_err(url_err_any)?;
                    diesel::delete(api_keys.filter(key.eq(&api_key_entry.key)))
                        .execute(&conn)
                        .map_err(url_err_any)
                }).await?.map_err(actix_web::error::ErrorInternalServerError)?;;

                Ok(HttpResponse::Ok().finish())
            } else {
                Ok(HttpResponse::NotFound().finish())
            }
        },
        _ => Ok(HttpResponse::MethodNotAllowed().finish())
    }
}