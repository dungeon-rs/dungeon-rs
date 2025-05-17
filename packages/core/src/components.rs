//! This module contains the ECS modules that make up the DungeonRS project structure.
//!
//! A typical DungeonRS map will be structured like this:
//! ```rust
//! use bevy::ecs::children;
//! use crate::core::components::*;
//! use bevy::prelude::*;
//!
//! (
//!     Project::new(Rect::new(-100., -100., 100., 100.)), // Always one, top level of the hierarchy
//!     Name::new("Example"),
//!     children![
//!         (
//!             Level, // There can be multiple levels
//!             Name::new("Default"),
//!             children![
//!                 (
//!                     Layer, // There can be multiple levels, the Z-axis determines the order.
//!                     Name::new("base"),
//!                     children![
//!                         // Here's where you'd place the things rendered.
//!                     ]
//!                 ),
//!             ]
//!         ),
//!     ]
//! )
//! ```

#[doc(hidden)]
mod layer;
#[doc(hidden)]
mod level;
#[doc(hidden)]
mod project;
#[doc(hidden)]
mod texture;

#[doc(inline)]
pub use {
    layer::Layer,
    level::Level,
    project::Project,
    texture::Texture,
};
