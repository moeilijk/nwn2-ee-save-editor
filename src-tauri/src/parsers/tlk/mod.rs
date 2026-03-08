pub mod error;
pub mod parser;
pub mod types;

pub use error::{SecurityLimits, TLKError, TLKResult};
pub use parser::load_multiple_files;
pub use types::{
    BatchMetrics, BatchStringResult, FileMetadata, ParserStatistics, SearchOptions, SearchResult,
    SerializableTLKParser, TLKHeader, TLKParser, TLKStringEntry,
};
