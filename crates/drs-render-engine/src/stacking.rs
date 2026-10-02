//! The stacking order every drawing of an Element shares: one depth unit per Element, counted up
//! through every Level and Layer of every Project in the order of their children.

use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::Children;
use bevy_ecs::query::With;
use bevy_ecs::system::{Query, SystemParam};
use drs_model::{Layer, Level, Project};

/// The Projects, Levels, and Layers the stacking order is read from.
#[derive(SystemParam)]
pub(crate) struct Stacking<'w, 's> {
    /// Each Project's Levels.
    projects: Query<'w, 's, (Entity, &'static Children), With<Project>>,
    /// Each Level's Layers.
    levels: Query<'w, 's, &'static Children, With<Level>>,
    /// Each Layer's Elements in stacking order.
    layers: Query<'w, 's, &'static Children, With<Layer>>,
}

impl Stacking<'_, '_> {
    /// Every Element with its Project and its depth, the first drawn first, so no two Elements
    /// share a depth and a later Element is drawn over an earlier one, whatever draws it.
    pub(crate) fn in_order(&self) -> Vec<Stacked> {
        let mut stacked = Vec::new();
        for (project, levels) in &self.projects {
            for &level in levels {
                let Ok(layers) = self.levels.get(level) else {
                    continue;
                };
                for &layer in layers {
                    let Ok(elements) = self.layers.get(layer) else {
                        continue;
                    };
                    for &element in elements {
                        #[expect(
                            clippy::cast_precision_loss,
                            reason = "far fewer Elements are drawn than f32 counts exactly"
                        )]
                        let depth = stacked.len() as f32;
                        stacked.push(Stacked {
                            project,
                            element,
                            depth,
                        });
                    }
                }
            }
        }
        stacked
    }
}

/// One Element's place in the stacking order.
pub(crate) struct Stacked {
    /// The Project it belongs to.
    pub project: Entity,
    /// The Element.
    pub element: Entity,
    /// The depth it is drawn at.
    pub depth: f32,
}
