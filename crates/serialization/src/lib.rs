#![doc = include_str!("../README.md")]
#![allow(clippy::missing_docs_in_private_items)]

mod deserialize;
mod error;
mod format;
mod serialize;

pub use deserialize::*;
pub use error::*;
pub use format::*;
pub use serde::{Deserialize, Serialize};
pub use serialize::*;
