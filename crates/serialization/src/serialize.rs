use crate::Serialize;
use crate::error::SerializationError;
use crate::format::SerializationFormat;
use anyhow::Error;
use std::io::Write;

/// Shorthand for `Result<T, SerializationError>`.
type Result<T> = std::result::Result<T, SerializationError>;

/// A simple wrapper function that calls [`serialize`] and writes the result to `writer`.
///
/// This method also calls `writer.flush()`
#[allow(clippy::missing_errors_doc)]
pub fn serialize_to<T>(
    subject: &T,
    format: &SerializationFormat,
    mut writer: impl Write,
) -> Result<()>
where
    T: ?Sized + Serialize,
{
    let binary = serialize(subject, format)?;
    writer
        .write_all(&binary)
        .map_err(SerializationError::from)?;

    writer.flush().map_err(SerializationError::from)?;

    Ok(())
}

/// Attempts to serialize `subject` into the specified format and returns the result as a `Vec<u8>`.
///
/// # Arguments
///
/// * `subject`: The data structure to serialize.
/// * `format`: A [`SerializationFormat`] that indicates which serialization format to use.
///
/// returns: [`Result<Vec<u8>>`]
///
/// # Errors
/// This method returns a [`SerializationError`] if any of the steps for serialization fails.
/// See [`SerializationError`] for specific details on each error scenario.
///
/// # Examples
///
/// ```
/// # use serde::*;
/// # use serialization::*;
///
/// # #[derive(Serialize)]
/// # struct Foo {
/// #     bar: String,
/// # }
///
/// # fn main() -> anyhow::Result<()> {
/// serialize(&Foo { bar: "baz".to_string() }, &SerializationFormat::JSON)?;
/// #     Ok(())
/// # }
/// ```
#[inline]
pub fn serialize<T>(subject: &T, format: &SerializationFormat) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    match format {
        SerializationFormat::JSON => serialize_json(subject),
        SerializationFormat::MessagePack => serialize_messagepack(subject),
        SerializationFormat::Toml => serialize_toml(subject),
    }
}

/// Attempts to serialize `subject` into JSON and returns the string as a `Vec<u8>`.
#[allow(clippy::missing_errors_doc)]
pub fn serialize_json<T>(subject: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    #[cfg(not(feature = "dev"))]
    let json = serde_json::to_string(subject)
        .map_err(|error| SerializationError::Serialize(Error::from(error)))?;
    #[cfg(feature = "dev")]
    let json = serde_json::to_string_pretty(subject)
        .map_err(|error| SerializationError::Serialize(Error::from(error)))?;

    Ok(json.into_bytes())
}

/// Attempts to serialize `subject` into `MessagePack`.
#[allow(clippy::missing_errors_doc)]
#[cfg_attr(not(feature = "msgpack"), allow(unused_variables))]
pub fn serialize_messagepack<T>(subject: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    #[cfg(not(feature = "msgpack"))]
    return Err(SerializationError::FormatUnavailable("msgpack"));

    #[cfg(feature = "msgpack")]
    rmp_serde::to_vec(subject).map_err(|error| SerializationError::Serialize(Error::from(error)))
}

#[allow(clippy::missing_errors_doc)]
#[cfg_attr(not(feature = "toml"), allow(unused_variables))]
pub fn serialize_toml<T>(subject: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    #[cfg(not(feature = "toml"))]
    return Err(SerializationError::FormatUnavailable("toml"));

    #[cfg(feature = "toml")]
    let toml = toml::to_string(subject)
        .map_err(|error| SerializationError::Serialize(Error::from(error)))?;
    #[cfg(all(feature = "toml", feature = "dev"))]
    let toml = toml::to_string_pretty(subject)
        .map_err(|error| SerializationError::Serialize(Error::from(error)))?;

    #[cfg(feature = "toml")]
    Ok(toml.into_bytes())
}
