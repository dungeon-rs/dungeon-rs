//! Contains small utility functions for DungeonRS.

use serde::Serialize;

/// Serializes the given [T] using the configured serializer.
pub fn serialize<T>(subject: &T) -> Result<Vec<u8>, String>
where
    T: Serialize,
{
    #[cfg(feature = "json")]
    return match serde_json::to_string_pretty(subject) {
        Ok(json) => Ok(json.into()),
        Err(error) => Err(error.to_string()),
    };

    #[allow(unreachable_code)]
    #[cfg(feature = "msgpack")]
    return match rmp_serde::to_vec(subject) {
        Ok(result) => Ok(result),
        Err(error) => Err(error.to_string()),
    };

    #[cfg(not(any(feature = "json", feature = "msgpack")))]
    compile_error!(
        "Either the `json` or `msgpack` feature must be enabled for serialization to work."
    );
}

/// Deserializes the given bytes into the specified type using the configured deserializer.
pub fn deserialize<T>(bytes: &[u8]) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    #[cfg(feature = "json")]
    return match std::str::from_utf8(bytes) {
        Ok(s) => match serde_json::from_str(s) {
            Ok(result) => Ok(result),
            Err(error) => Err(error.to_string()),
        },
        Err(err) => Err(err.to_string()),
    };

    #[allow(unreachable_code)]
    #[cfg(feature = "msgpack")]
    return match rmp_serde::from_slice(bytes) {
        Ok(msgpack) => Ok(msgpack),
        Err(error) => return Err(error.to_string()),
    };

    #[cfg(not(any(feature = "json", feature = "msgpack")))]
    compile_error!(
        "Either the `json` or `msgpack` feature must be enabled for deserialization to work."
    );
}
