use config::ConfigError;
use crate::model::db::{DATABASE_URL, get_db_path};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Config {
    pub hostname : String
}

pub fn load_conf() -> Result<Config, ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::Environment::with_prefix("URL"))
        .add_source(config::File::with_name(format!("{}/{}", get_db_path(), "config.yaml").as_str()).required(false))
        .build()
        .unwrap();

    let config : Config = settings.try_deserialize()?;

    Ok(config)
}