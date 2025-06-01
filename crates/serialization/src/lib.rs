#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic, clippy::suspicious, clippy::complexity)]

mod deserialize;
mod error;
mod format;
mod serialize;

pub use deserialize::*;
pub use error::*;
pub use format::*;
pub use serde::{Deserialize, Serialize};
pub use serialize::*;
