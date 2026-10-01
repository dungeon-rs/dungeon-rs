//! Place Element: a Prop of a chosen Asset on a Layer, and its undo.

use crate::AuthoringError;
use bevy_ecs::entity::Entity;
use bevy_ecs::error::BevyError;
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::world::World;
use bevy_math::Vec2;
use drs_history::{ReversibleCommand, Target};
use drs_library_access::load_asset;
use drs_model::{
    AssetFolder, AssetFolderReference, AssetReference, AssetReferences, Element, ElementId, Grid,
    Layer, PROP, PlaceElement, Project, Prop,
};
use unicode_normalization::UnicodeNormalization;

/// The Project a Layer belongs to: the nearest ancestor carrying [`Project`].
///
/// # Errors
///
/// [`AuthoringError::NoProject`] when no ancestor is a Project.
fn project_of(world: &World, layer: Entity) -> Result<Entity, AuthoringError> {
    let mut current = layer;
    while let Some(parent) = world.get::<ChildOf>(current).map(ChildOf::parent) {
        if world.get::<Project>(parent).is_some() {
            return Ok(parent);
        }
        current = parent;
    }
    Err(AuthoringError::NoProject)
}

/// The recorded step: the Element spawned on top of its Layer, keeping its identity so that
/// redo puts it back exactly.
///
/// The Asset Reference row the Element refers to, and the Asset Folder row it comes from, are
/// recorded on the first application and never removed: undoing the placement leaves them in the
/// table, and placing the Asset again reuses them. Pruning rows no Element uses is a later
/// concern.
struct Place {
    /// The Project whose Asset Reference table records the Asset.
    project: Entity,
    /// The Layer to place on.
    layer: Entity,
    /// The centre of the Element in Grid cells.
    position: Vec2,
    /// The Element's natural size in Grid cells.
    size: Vec2,
    /// What the Project records about the Asset.
    reference: AssetReference,
    /// What the Project records about the Asset's folder.
    folder: AssetFolderReference,
    /// The identity the Element keeps through undo and redo.
    element: ElementId,
}

impl ReversibleCommand for Place {
    fn apply(&mut self, world: &mut World) -> Result<(), BevyError> {
        if world.get_entity(self.layer).is_err() {
            return Err(AuthoringError::NotALayer.into());
        }
        let row = world
            .get_mut::<AssetReferences>(self.project)
            .ok_or(AuthoringError::NoProject)?
            .record(self.reference.clone(), self.folder.clone())?;
        let entity = world
            .spawn((
                Element {
                    kind: PROP,
                    position: self.position,
                    size: self.size,
                },
                Prop { asset: row },
                self.element,
            ))
            .id();
        // A redone Place is always last too: every step after it has been undone first, so the
        // Layer holds exactly the Elements it held when the Prop was first placed on top.
        world
            .get_entity_mut(self.layer)
            .map_err(|_| AuthoringError::NotALayer)?
            .add_child(entity);
        Ok(())
    }

    fn revert(&mut self, world: &mut World) -> Result<(), BevyError> {
        let entity = self.element.entity(world)?;
        world.despawn(entity);
        Ok(())
    }
}

/// Place Element: resolves the chosen Asset, reads what the Project must record about it, and
/// places a Prop of it on top of the Layer, centred on the given position, as one history step.
///
/// # Errors
///
/// [`AuthoringError::NotALayer`] or [`AuthoringError::NoProject`] when the Layer is not one or
/// belongs to no Project, [`AuthoringError::UnknownFolder`] or [`AuthoringError::UnknownAsset`]
/// when the chosen Asset is not indexed, [`AuthoringError::Library`] when its file cannot be
/// read, or [`AuthoringError::History`] when the step could not be recorded.
pub(crate) fn place_element(
    world: &mut World,
    command: &PlaceElement,
) -> Result<(), AuthoringError> {
    if world.get::<Layer>(command.layer).is_none() {
        return Err(AuthoringError::NotALayer);
    }
    let project = project_of(world, command.layer)?;
    let pixels_per_cell = world.get::<Grid>(project).map_or_else(
        || Grid::default().pixels_per_cell,
        |grid| grid.pixels_per_cell,
    );

    let folder = world
        .query::<&AssetFolder>()
        .iter(world)
        .find(|folder| folder.key == command.asset.folder)
        .cloned()
        .ok_or_else(|| AuthoringError::UnknownFolder(command.asset.folder.clone()))?;
    let indexed = folder
        .assets
        .iter()
        .find(|asset| asset.place == command.asset.place)
        .ok_or_else(|| AuthoringError::UnknownAsset {
            folder: folder.name.clone(),
            place: command.asset.place.clone(),
        })?;
    let loaded = load_asset(&folder.path, &folder.key, &indexed.place)?;

    #[expect(
        clippy::cast_precision_loss,
        reason = "cells per image are far below the range where f32 loses whole numbers"
    )]
    let size = loaded.pixel_size.as_vec2() / pixels_per_cell as f32;
    let reference = AssetReference {
        folder: folder.name.clone(),
        place: indexed.place.nfc().collect(),
        name: indexed.name.clone(),
        kind: indexed.kind.clone(),
        fingerprint: loaded.fingerprint,
        byte_size: loaded.byte_size,
        pixel_size: Some(loaded.pixel_size),
    };
    crate::record_step(
        world,
        Place {
            project,
            layer: command.layer,
            position: command.position,
            size,
            reference,
            folder: AssetFolderReference {
                name: folder.name,
                version: folder.version,
            },
            element: ElementId::new(),
        },
    )
}
