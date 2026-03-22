use crate::handlers::{
    bad_request_error, forbidden_error, internal_error, not_found_error, unauthorized_error,
};
use dropshot::HttpError;
use rusqlite::Result as SqliteResult;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    BadRequest(String),

    #[error("{0}")]
    Forbidden(String),

    #[error("{0}")]
    Unauthorized(String),

    #[error("{0}")]
    Internal(String),

    #[error("{context}: {source}")]
    Db {
        #[source]
        source: rusqlite::Error,
        context: String,
    },
}

impl From<AppError> for HttpError {
    fn from(err: AppError) -> Self {
        match err {
            AppError::NotFound(msg) => not_found_error(msg),
            AppError::BadRequest(msg) => bad_request_error(msg),
            AppError::Forbidden(msg) => forbidden_error(msg),
            AppError::Unauthorized(msg) => unauthorized_error(msg),
            AppError::Internal(msg) => internal_error(msg),
            AppError::Db {
                ref context,
                ref source,
            } => {
                eprintln!("ERROR: {}: {}", context, source);
                internal_error(context.clone())
            }
        }
    }
}

/// Extension trait for converting `SqliteResult<T>` to `Result<T, AppError>` with context.
pub trait DbResultExt<T> {
    fn db_context(self, ctx: &str) -> Result<T, AppError>;
}

impl<T> DbResultExt<T> for SqliteResult<T> {
    fn db_context(self, ctx: &str) -> Result<T, AppError> {
        self.map_err(|e| AppError::Db {
            source: e,
            context: ctx.to_string(),
        })
    }
}

/// Extension trait for converting `Option<T>` to `Result<T, AppError>`.
pub trait OptionExt<T> {
    fn or_not_found(self, msg: impl Into<String>) -> Result<T, AppError>;
}

impl<T> OptionExt<T> for Option<T> {
    fn or_not_found(self, msg: impl Into<String>) -> Result<T, AppError> {
        self.ok_or_else(|| AppError::NotFound(msg.into()))
    }
}
