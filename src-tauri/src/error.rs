use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Path not found: {0}")]
    PathNotFound(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Security violation: {0}")]
    Security(String),

    #[error("File size exceeded: {size} bytes (max: {max} bytes)")]
    FileSizeExceeded { size: usize, max: usize },

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Encoding error: {0}")]
    Encoding(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

pub trait ResultExt<T, E> {
    fn map_app_err<F: FnOnce(E) -> AppError>(self, f: F) -> AppResult<T>;
}

impl<T, E: fmt::Display> ResultExt<T, E> for Result<T, E> {
    fn map_app_err<F: FnOnce(E) -> AppError>(self, f: F) -> AppResult<T> {
        self.map_err(f)
    }
}

#[macro_export]
macro_rules! define_parser_error {
    (
        $name:ident {
            $(
                $variant:ident $( { $($field:ident : $field_ty:ty),* } )? $( ( $($tuple_ty:ty),* ) )? => $msg:literal
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, thiserror::Error)]
        pub enum $name {
            $(
                #[error($msg)]
                $variant $( { $($field : $field_ty),* } )? $( ( $($tuple_ty),* ) )?,
            )*

            #[error("I/O error: {0}")]
            Io(#[from] std::io::Error),

            #[error("Serialization error: {0}")]
            Serialization(String),

            #[error("Deserialization error: {0}")]
            Deserialization(String),

            #[error("Security violation: {0}")]
            SecurityViolation(String),
        }

        impl From<$name> for $crate::error::AppError {
            fn from(err: $name) -> $crate::error::AppError {
                $crate::error::AppError::Parse(err.to_string())
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_display() {
        let err = AppError::FileSizeExceeded {
            size: 1000,
            max: 500,
        };
        assert!(err.to_string().contains("1000"));
        assert!(err.to_string().contains("500"));
    }

    #[test]
    fn test_app_error_serialize() {
        let err = AppError::NotFound("test.2da".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("test.2da"));
    }
}
