use diesel::prelude::*;
use diesel::{ConnectionResult, SqliteConnection};

const DATABASE_URL : &str = "db";

pub fn get_db_conn() -> ConnectionResult<SqliteConnection> {
    let data_dir_path = std::env::var("URL_DATA_DIR").unwrap_or_else(|_| {"./".to_owned()});
    let db_file_path = format!("{}/{}", data_dir_path, DATABASE_URL);
    diesel::sqlite::SqliteConnection::establish(&db_file_path)
}
