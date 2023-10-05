use actix_web::{error::ResponseError, http::StatusCode};

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials.")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for AuthError {
    fn status_code(&self) -> StatusCode {
        match self {
            AuthError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthError::InvalidCredentials(_) => StatusCode::UNAUTHORIZED,
        }
    }
}
