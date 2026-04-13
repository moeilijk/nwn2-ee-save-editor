use thiserror::Error;

#[derive(Error, Debug)]
pub enum Gr2Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid GR2 magic: file is not a valid Granny2 file")]
    InvalidMagic,

    #[error("Invalid GR2 version: expected 6, found {found}")]
    InvalidVersion { found: u32 },

    #[error("Unsupported compression type: {0} (only 0=none and 2=Oodle1 are supported)")]
    UnsupportedCompression(u32),

    #[error("Decompression failed: {message}")]
    DecompressFailed { message: String },

    #[error("Invalid section index: {index} (max {max})")]
    InvalidSectionIndex { index: u32, max: u32 },

    #[error("Invalid offset: {offset} exceeds data size {size}")]
    InvalidOffset { offset: usize, size: usize },

    #[error("Security violation: {message}")]
    SecurityViolation { message: String },

    #[error("No skeleton data found in GR2 file")]
    NoSkeleton,

    #[error("No animations found in GR2 file")]
    NoAnimations,

    #[error("Unsupported curve format: {format}")]
    UnsupportedCurveFormat { format: u32 },

    #[error("Unexpected end of data at offset {offset}")]
    UnexpectedEof { offset: usize },
}

pub type Gr2Result<T> = std::result::Result<T, Gr2Error>;
