use std::fmt::{Display, Formatter};
use actix_web::{HttpResponse, ResponseError};
use http::StatusCode;
use thiserror::Error;
use serde::{Serialize};
use thiserror::private::DisplayAsDisplay;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Catch-all error type")]
    Any(Box<dyn std::error::Error + Send>),
    #[error("Request error")]
    RequestError(Box<dyn std::error::Error + Send>),
}

impl Error {
    pub fn err_msg(&self) -> String {
        match self {
            Error::Any(err) => {
                err.to_string()
            }
            Error::RequestError(err) => {
                err.to_string()
            }
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    error: String,
    message: String,
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            code: status_code.as_u16(),
            message: self.err_msg(),
            error: self.to_string(),
        };
        HttpResponse::build(status_code).json(error_response)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Error::Any(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::RequestError(_) => StatusCode::BAD_REQUEST
        }
    }
}

pub fn url_err_any<E : std::error::Error + 'static + Send>(err : E) -> Error {
    Error::Any(Box::new(err))
}

pub fn url_err_request<E : std::error::Error + 'static + Send>(err : E) -> Error {
    Error::RequestError(Box::new(err))
}