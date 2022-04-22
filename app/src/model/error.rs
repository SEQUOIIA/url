use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Catch-all error type")]
    Any(Box<dyn std::error::Error + Send>)
}

pub fn url_err_any<E : std::error::Error + 'static + Send>(err : E) -> Error {
    Error::Any(Box::new(err))
}