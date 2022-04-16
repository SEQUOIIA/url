use log::info;

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod model;
mod web;
mod schema;

use diesel_migrations::{embed_migrations};

embed_migrations!("../migrations");


fn main() {
    setup_tracing(Some(tracing::Level::TRACE));
    let app_version = env!("CARGO_PKG_VERSION");
    info!("Launching url v{} service", app_version);

    // db setup
    {
        let conn = model::db::get_db_conn().unwrap();
        embedded_migrations::run_with_output(&conn, &mut std::io::stdout());
    }

    web::start_server();
}

fn setup_tracing(log_level : Option<tracing::Level>) {
    let max_level = log_level.unwrap_or(tracing::Level::TRACE);
    let filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive(format!("seqtf_url={}", max_level.as_str().to_lowercase()).parse().unwrap())
        .add_directive("reqwest=info".parse().unwrap())
        .add_directive("mio=info".parse().unwrap())
        .add_directive("want=info".parse().unwrap())
        .add_directive("actix_web=info".parse().unwrap())
        .add_directive("hyper=info".parse().unwrap());


    let subscriber = tracing_subscriber::fmt()
        .with_max_level(max_level)
        .with_env_filter(filter)
        .finish();
    tracing_log::LogTracer::init().unwrap();
    tracing::subscriber::set_global_default(subscriber).unwrap();
}

fn get_trace_level(input : &str) -> Option<tracing::Level> {
    return match input.to_uppercase().as_str() {
        "TRACE" => Some(tracing::Level::TRACE),
        "DEBUG" => Some(tracing::Level::DEBUG),
        "INFO" => Some(tracing::Level::INFO),
        "WARN" => Some(tracing::Level::WARN),
        "ERROR" => Some(tracing::Level::ERROR),
        _ => None
    }
}
