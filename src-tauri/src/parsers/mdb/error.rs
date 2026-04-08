use thiserror::Error;

#[derive(Error, Debug)]
pub enum MdbError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid MDB signature: expected 'NWN2', found '{found}'")]
    InvalidSignature { found: String },

    #[error("Invalid MDB version: {major}.{minor}")]
    InvalidVersion { major: u16, minor: u16 },

    #[error("Invalid packet type: '{found}'")]
    InvalidPacketType { found: String },

    #[error("Invalid packet offset: {offset} exceeds file size {file_size}")]
    InvalidOffset { offset: u32, file_size: usize },

    #[error("Security violation: {message}")]
    SecurityViolation { message: String },

    #[error("Unexpected end of data at offset {offset}, needed {needed} more bytes")]
    UnexpectedEof { offset: usize, needed: usize },
}

pub type MdbResult<T> = std::result::Result<T, MdbError>;
