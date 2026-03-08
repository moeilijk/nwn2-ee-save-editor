pub mod gff;
pub mod tda;
pub mod tlk;
pub mod erf;
pub mod xml;

pub use gff::{GffParser, GffFieldType, GffValue};
pub use tda::TDAParser;
pub use tlk::TLKParser;
pub use erf::ErfParser;
pub use xml::RustXmlParser;
