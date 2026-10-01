#![doc = include_str!("../README.md")]

mod generic;
mod stack;

pub use generic::{SetField, Snapshot, Target};
pub use stack::{History, HistoryPlugin, apply, redo, undo};

use bevy_ecs::error::BevyError;
use bevy_ecs::world::World;

/// A command that can be carried out and taken back on a [`World`].
///
/// `apply` is also used to redo a command, so it must repeat exactly what the first
/// application did; `revert` must leave the `World` as it was before `apply`.
pub trait ReversibleCommand: Send + Sync + 'static {
    /// Carries the command out, or carries it out again after it was reverted.
    ///
    /// # Errors
    ///
    /// Whatever the command cannot do; nothing is recorded when the first application fails.
    fn apply(&mut self, world: &mut World) -> Result<(), BevyError>;

    /// Takes the command back.
    ///
    /// # Errors
    ///
    /// Whatever the command cannot undo; the step then stays where it was in the history.
    fn revert(&mut self, world: &mut World) -> Result<(), BevyError>;
}

/// What can go wrong inside the history itself or its generic commands.
#[derive(Debug, thiserror::Error)]
pub enum HistoryError {
    /// The `World` carries no [`History`] resource; the [`HistoryPlugin`] was not added.
    #[error("the World has no History")]
    NoHistory,
    /// The entity a command works on cannot be found.
    #[error("the target of the command does not exist")]
    MissingTarget,
    /// The component type is not registered with `ReflectComponent` type data.
    #[error("component `{0}` is not registered for reflection")]
    UnregisteredComponent(String),
    /// The target entity has no component of the named type.
    #[error("the entity has no `{0}` component")]
    MissingComponent(String),
    /// A reflect path could not be parsed or followed.
    #[error("reflect path `{path}`: {reason}")]
    Path {
        /// The path as written.
        path: String,
        /// Why it failed.
        reason: String,
    },
}
