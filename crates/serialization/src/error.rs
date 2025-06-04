//! Contains all errors defined and used by the `serialization` crate.
use thiserror::Error;

/// Represents potential serialisation errors.
#[derive(Error, Debug)]
pub enum SerializationError {
    /// Error when the selected serialisation format is unavailable.
    ///
    /// This is usually due to a missing feature flag.
    #[error("The selected serialisation format is unavailable, enable feature '{}' to use.", .0)]
    FormatUnavailable(&'static str),
    /// Some (de)serialisation methods require I/O, if any of those I/O operations fails, this error
    /// is returned.
    #[error("An error occurred while writing to output: {0}")]
    IO(#[from] std::io::Error),
    /// An error occurred while attempting to serialize, this is mostly a wrapper for the underlying
    /// error thrown by the serialisation libraries.
    #[error("An error occurred while serialising: {0}")]
    Serialize(anyhow::Error),
    /// An error occurred while attempting to deserialize, this is mostly a wrapper for the underlying
    /// error thrown by the serialisation libraries.
    #[error("An error occurred while deserialising: {0}")]
    Deserialize(anyhow::Error),
}
