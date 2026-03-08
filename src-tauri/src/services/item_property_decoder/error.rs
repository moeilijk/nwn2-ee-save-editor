use thiserror::Error;

#[derive(Debug, Error)]
pub enum ItemPropertyError {
    #[error("Property type not found: {id}")]
    PropertyTypeNotFound { id: u32 },

    #[error("Invalid property data: {0}")]
    InvalidPropertyData(String),

    #[error("Table not found: {name}")]
    TableNotFound { name: String },

    #[error("String reference not found: {str_ref}")]
    StringNotFound { str_ref: i32 },

    #[error("Decoding failed for property {property_id}: {reason}")]
    DecodingFailed { property_id: u32, reason: String },

    #[error("Metadata generation failed: {0}")]
    MetadataFailed(String),

    #[error("Resource manager error: {0}")]
    ResourceManager(String),
}

impl serde::Serialize for ItemPropertyError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<ItemPropertyError> for crate::error::AppError {
    fn from(err: ItemPropertyError) -> crate::error::AppError {
        crate::error::AppError::Parse(err.to_string())
    }
}

pub type ItemPropertyResult<T> = Result<T, ItemPropertyError>;
