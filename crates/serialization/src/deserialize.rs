use crate::Deserialize;
use crate::error::SerializationError;
use crate::format::SerializationFormat;
use anyhow::Error;

/// Shorthand for `Result<T, SerializationError>`.
type Result<T> = std::result::Result<T, SerializationError>;

/// Attempts to deserialize `subject` using `format` into [`T`].
///
/// # Arguments
///
/// * `subject`: The `Vec<u8>` that represents [`T`] in binary format.
/// * `format`: The format to use when deserializing `subject`.
///
/// returns: [`Result<T>`]
///
/// # Errors
/// This method returns a [`SerializationError`] if any of the steps for deserialization fails.
/// See [`SerializationError`] for specific details on each error scenario.
///
/// # Examples
///
/// ```
/// # use serde::*;
/// # use serialization::*;
///
/// # #[derive(Serialize, Deserialize)]
/// # struct Foo {
/// #     bar: String,
/// # }
///
/// # fn main() -> anyhow::Result<()> {
/// let json = b"{ \"bar\": \"baz\" }";
/// deserialize(json, &SerializationFormat::JSON)?;
/// #     Ok(())
/// # }
#[inline]
pub fn deserialize<'a, T>(subject: &'a [u8], format: &SerializationFormat) -> Result<T>
where
    T: Deserialize<'a>,
{
    match format {
        SerializationFormat::JSON => deserialize_json(subject),
        SerializationFormat::MessagePack => deserialize_messagepack(subject),
        SerializationFormat::Toml => deserialize_toml(subject),
    }
}

/// Attempts to deserialize `subject` using JSON.
#[allow(clippy::missing_errors_doc)]
pub fn deserialize_json<'a, T: Deserialize<'a>>(subject: &'a [u8]) -> Result<T> {
    serde_json::from_slice(subject)
        .map_err(|error| SerializationError::Deserialize(Error::from(error)))
}

/// Attempts to deserialize `subject` using [MessagePack](https://msgpack.org/).
#[allow(clippy::missing_errors_doc)]
#[cfg_attr(not(feature = "msgpack"), allow(unused_variables))]
pub fn deserialize_messagepack<'a, T: Deserialize<'a>>(subject: &'a [u8]) -> Result<T> {
    #[cfg(not(feature = "msgpack"))]
    return Err(SerializationError::FormatUnavailable("msgpack"));

    #[cfg(feature = "msgpack")]
    rmp_serde::from_slice(subject)
        .map_err(|error| SerializationError::Deserialize(Error::from(error)))
}

/// Attempts to deserialize `subject` using [TOML](https://toml.io/).
#[allow(clippy::missing_errors_doc)]
#[cfg_attr(not(feature = "toml"), allow(unused_variables))]
pub fn deserialize_toml<'a, T>(subject: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    #[cfg(not(feature = "toml"))]
    return Err(SerializationError::FormatUnavailable("toml"));

    #[cfg(feature = "toml")]
    {
        let str = String::from_utf8_lossy(subject);
        let deserializer = toml::Deserializer::new(&str);

        T::deserialize(deserializer)
            .map_err(|error| SerializationError::Deserialize(Error::from(error)))
    }
}
