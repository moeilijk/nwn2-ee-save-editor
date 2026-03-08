//! Character error types - consolidated from 11 separate manager error modules.

use serde::Serialize;
use thiserror::Error;

/// Unified error type for Character operations.
///
/// Consolidates error handling from the previous 11-manager architecture
/// into a single, simpler error type.
#[derive(Debug, Error)]
pub enum CharacterError {
    #[error("Required field missing: {field}")]
    FieldMissing { field: &'static str },

    #[error("Value {value} out of range for {field} (valid: {min}-{max})")]
    OutOfRange {
        field: &'static str,
        value: i32,
        min: i32,
        max: i32,
    },

    #[error("Validation failed for {field}: {message}")]
    ValidationFailed {
        field: &'static str,
        message: String,
    },

    #[error("Not found: {entity} with ID {id}")]
    NotFound { entity: &'static str, id: i32 },

    #[error("Already exists: {entity} with ID {id}")]
    AlreadyExists { entity: &'static str, id: i32 },

    #[error("Protected: {entity} ID {id} cannot be modified")]
    Protected { entity: &'static str, id: i32 },

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Feat {0} not found")]
    FeatNotFound(i32),

    #[error("Feat {0} already exists")]
    FeatAlreadyExists(i32),

    #[error("Game data table not found: {0}")]
    TableNotFound(String),
}

impl Serialize for CharacterError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type CharacterResult<T> = Result<T, CharacterError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = CharacterError::OutOfRange {
            field: "Str",
            value: 55,
            min: 3,
            max: 50,
        };
        assert!(err.to_string().contains("55"));
        assert!(err.to_string().contains("3-50"));
    }

    #[test]
    fn test_error_serialize() {
        let err = CharacterError::NotFound {
            entity: "Race",
            id: 42,
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("42"));
    }
}
