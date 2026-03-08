use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlayerInfoParseError {
    #[error("Invalid file format: {0}")]
    InvalidFormat(String),

    #[error("Unexpected end of file at position {position}")]
    UnexpectedEof { position: u64 },

    #[error("Invalid string at position {position}: {reason}")]
    InvalidString { position: u64, reason: String },

    #[error("Invalid class entry at index {index}")]
    InvalidClassEntry { index: usize },

    #[error("Encoding error: {0}")]
    Encoding(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl serde::Serialize for PlayerInfoParseError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<PlayerInfoParseError> for crate::error::AppError {
    fn from(err: PlayerInfoParseError) -> crate::error::AppError {
        crate::error::AppError::Parse(err.to_string())
    }
}

pub type PlayerInfoResult<T> = Result<T, PlayerInfoParseError>;
