//! For more information on Bevy's state machine implementation read
//! [their documentation](https://docs.rs/bevy/latest/bevy/state).
//!
//! DungeonRS states are mainly used to coordinate between systems that may need to run conditionally.
//! For example, we don't want the UI to be shown during an export (as it would be rendered as well)
//! or we want to display loading messages/screens while the application is performing long-running
//! operations.
//!
//! Important is to remember we don't use the state machine to communicate information between systems,
//! that's what we use the event system for, state machine should be treated as a way to conditionally
//! turn certain functionality on/off (e.g. UI, interactivity, ...) based on the state of other systems.

use bevy::prelude::States;

/// Represents the root level state of DungeonRS.
/// Every other state should be defined as a substate of this [DungeonRsState].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum DungeonRsState {
    #[default]
    Splash,
}
