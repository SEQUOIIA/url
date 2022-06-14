use thiserror::Error;
use crate::config::ConfigError;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    // #[error("unable to get status of entry")]
    // UnableToGetStatusOfEntry(),
    #[error("custom error: {0:?}")]
    CustomError(Box<dyn std::error::Error>),
    #[error("error: {0}")]
    Message(String),
    #[error("config error: {0:?}")]
    ConfigErr(ConfigError),
}