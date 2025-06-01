use thiserror::Error;

/// Represents potential serialization errors.
#[derive(Error, Debug)]
pub enum SerializationError {
    /// Error when the selected serialization format is unavailable.
    ///
    /// This is usually due to a missing feature flag.
    #[error("The selected serialization format is unavailable, enable feature '{}' to use.", .0)]
    FormatUnavailable(&'static str),
    /// Some (de)serialization methods require I/O, if any of those I/O operations fails, this error
    /// is returned.
    #[error("An error occurred while writing to output: {0}")]
    IO(#[from] std::io::Error),
    /// An error occurred while attempting to serialize, this is mostly a wrapper for the underlying
    /// error thrown by the serialization libraries.
    #[error("An error occurred while serializing: {0}")]
    Serialize(anyhow::Error),
    /// An error occurred while attempting to deserialize, this is mostly a wrapper for the underlying
    /// error thrown by the serialization libraries.
    #[error("An error occurred while deserializing: {0}")]
    Deserialize(anyhow::Error),
}
