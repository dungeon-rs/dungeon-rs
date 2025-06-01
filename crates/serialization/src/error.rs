use thiserror::Error;

/// Represents potential serialization errors.
#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("The selected serialization format is unavailable, enable feature '{}' to use.", .0)]
    FormatUnavailable(&'static str),
    #[error("An error occurred while writing to output: {0}")]
    IO(#[from] std::io::Error),
    #[error("An error occurred while serializing: {0}")]
    Serialize(anyhow::Error),
    #[error("An error occurred while deserializing: {0}")]
    Deserialize(anyhow::Error),
}
