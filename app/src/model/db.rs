use diesel::prelude::*;
use diesel::{ConnectionResult, SqliteConnection};

pub const DATABASE_URL : &str = "db";

pub fn get_db_path() -> String {
    std::env::var("URL_DATA_DIR").unwrap_or_else(|_| {"./".to_owned()})
}

pub fn get_db_conn() -> ConnectionResult<SqliteConnection> {
    let db_file_path = format!("{}/{}", get_db_path(), DATABASE_URL);
    diesel::sqlite::SqliteConnection::establish(&db_file_path)
}
