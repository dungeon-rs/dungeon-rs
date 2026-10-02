//! Place Element: a Prop or a Portal of a chosen Asset on a Layer, and its undo.

use crate::AuthoringError;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::error::BevyError;
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::world::World;
use bevy_math::Vec2;
use drs_history::{ReversibleCommand, Target};
use drs_library_access::load_asset;
use drs_model::{
    AssetAddress, AssetFolder, AssetFolderReference, AssetReference, AssetReferences, Element,
    ElementId, Grid, IndexedAsset, Layer, PORTAL, PROP, PlaceElement, Placement, Portal, Project,
    Prop, Room, Wall,
};
use std::path::PathBuf;
use unicode_normalization::UnicodeNormalization;

/// The Project a Layer belongs to: the nearest ancestor carrying [`Project`].
///
/// # Errors
///
/// [`AuthoringError::NoProject`] when no ancestor is a Project.
pub(crate) fn project_of(world: &World, layer: Entity) -> Result<Entity, AuthoringError> {
    crate::ancestor(
        layer,
        |child| world.get::<ChildOf>(child).map(ChildOf::parent),
        |parent| world.get::<Project>(parent).is_some(),
    )
    .ok_or(AuthoringError::NoProject)
}

/// What an Element placed from an Asset is spawned as.
pub(crate) enum Spawned {
    /// A Prop.
    Prop,
    /// A Portal as it stands, its Asset Reference row set to the one the Project records the
    /// Asset in when the step is applied.
    Portal(Portal),
}

/// The recorded step: the Element spawned on top of its Layer, keeping its identity so that
/// redo puts it back exactly.
///
/// The Asset Reference row the Element refers to, and the Asset Folder row it comes from, are
/// recorded on the first application and never removed: undoing the placement leaves them in the
/// table, and placing the Asset again reuses them. Pruning rows no Element uses is a later
/// concern.
pub(crate) struct Place {
    /// The Layer to place on.
    pub(crate) layer: Entity,
    /// The centre of the Element in Grid cells.
    pub(crate) position: Vec2,
    /// The Asset as resolved, with the Element's natural size.
    pub(crate) resolved: Resolved,
    /// What the Element is spawned as.
    pub(crate) spawned: Spawned,
    /// The identity the Element keeps through undo and redo.
    pub(crate) element: ElementId,
}

impl ReversibleCommand for Place {
    fn apply(&mut self, world: &mut World) -> Result<(), BevyError> {
        if world.get_entity(self.layer).is_err() {
            return Err(AuthoringError::NotALayer.into());
        }
        let resolved = &self.resolved;
        let row = world
            .get_mut::<AssetReferences>(resolved.project)
            .ok_or(AuthoringError::NoProject)?
            .record(resolved.reference.clone(), resolved.folder.clone())?;
        match &self.spawned {
            Spawned::Prop => spawn_on_top(
                world,
                self.layer,
                (
                    Element {
                        kind: PROP,
                        position: self.position,
                        size: resolved.size,
                    },
                    Prop { asset: row },
                    self.element,
                ),
            ),
            Spawned::Portal(portal) => spawn_on_top(
                world,
                self.layer,
                (
                    Element {
                        kind: PORTAL,
                        position: self.position,
                        size: resolved.size,
                    },
                    Portal {
                        asset: row,
                        ..portal.clone()
                    },
                    self.element,
                ),
            ),
        }
    }

    fn revert(&mut self, world: &mut World) -> Result<(), BevyError> {
        take_off(world, self.element)
    }
}

/// Spawns an Element of `components`, its identity among them, as the last child of `layer`, on
/// top of the Layer's stacking order.
///
/// A redone placement is always last too: every step after it has been undone first, so the
/// Layer holds exactly the Elements it held when the Element was first placed on top.
///
/// # Errors
///
/// [`AuthoringError::NotALayer`] when the Layer is gone, before anything is spawned.
pub(crate) fn spawn_on_top(
    world: &mut World,
    layer: Entity,
    components: impl Bundle,
) -> Result<(), BevyError> {
    if world.get_entity(layer).is_err() {
        return Err(AuthoringError::NotALayer.into());
    }
    let entity = world.spawn(components).id();
    world
        .get_entity_mut(layer)
        .map_err(|_| AuthoringError::NotALayer)?
        .add_child(entity);
    Ok(())
}

/// Spawns an Element of `components`, its identity among them, as the first child of `layer`,
/// beneath every Element on the Layer.
///
/// A redone step that spawned beneath is first again for the same reason a redone placement is
/// last: every step after it has been undone first.
///
/// # Errors
///
/// [`AuthoringError::NotALayer`] when the Layer is gone, before anything is spawned.
pub(crate) fn spawn_beneath(
    world: &mut World,
    layer: Entity,
    components: impl Bundle,
) -> Result<(), BevyError> {
    if world.get_entity(layer).is_err() {
        return Err(AuthoringError::NotALayer.into());
    }
    let entity = world.spawn(components).id();
    world
        .get_entity_mut(layer)
        .map_err(|_| AuthoringError::NotALayer)?
        .insert_children(0, &[entity]);
    Ok(())
}

/// Takes a placed Element off its Layer again, by its identity: the revert of a placement.
///
/// # Errors
///
/// The history's error when no Element carries the identity.
pub(crate) fn take_off(world: &mut World, element: ElementId) -> Result<(), BevyError> {
    let entity = element.entity(world)?;
    world.despawn(entity);
    Ok(())
}

/// Place Element: places a Prop or a Portal of the chosen Asset, or a Wall or a Room through the
/// given points, on top of the Layer, as one history step.
///
/// # Errors
///
/// [`AuthoringError::NotALayer`] when the Layer is not one, and whatever placing the Prop, the
/// Portal, the Wall, or the Room reports.
pub(crate) fn place_element(
    world: &mut World,
    command: &PlaceElement,
) -> Result<(), AuthoringError> {
    if world.get::<Layer>(command.layer).is_none() {
        return Err(AuthoringError::NotALayer);
    }
    match &command.placement {
        Placement::Prop { position, asset } => place_prop(world, command.layer, *position, asset),
        Placement::Portal {
            position,
            asset,
            anchor,
        } => crate::portal::place_portal(world, command.layer, *position, asset, *anchor),
        Placement::Wall {
            points,
            thickness,
            colour,
        } => crate::outline::place_outline(
            world,
            command.layer,
            Wall::straight(points.clone(), *thickness, *colour),
        ),
        Placement::Room {
            points,
            thickness,
            wall_colour,
            floor_colour,
        } => crate::outline::place_outline(
            world,
            command.layer,
            Room::straight(points.clone(), *thickness, *wall_colour, *floor_colour),
        ),
    }
}

/// An Asset as a placement resolves it: the Project that records it, what the Project records
/// about it and its folder, and the natural size of an Element showing it.
pub(crate) struct Resolved {
    /// The Project whose Asset Reference table records the Asset.
    pub(crate) project: Entity,
    /// The Element's natural size in Grid cells: the image's pixel size over the Grid's pixels
    /// per cell.
    pub(crate) size: Vec2,
    /// What the Project records about the Asset.
    pub(crate) reference: AssetReference,
    /// What the Project records about the Asset's folder.
    pub(crate) folder: AssetFolderReference,
}

/// A chosen Asset as its Asset Folder indexes it, with where the folder lies and what the Project
/// records about the folder.
pub(crate) struct Indexed {
    /// The folder's path on this device.
    path: PathBuf,
    /// What the Project records about the folder.
    pub(crate) folder: AssetFolderReference,
    /// The Asset's entry in the folder's index.
    pub(crate) asset: IndexedAsset,
}

/// Finds a chosen Asset in its Asset Folder's index.
///
/// # Errors
///
/// [`AuthoringError::UnknownFolder`] or [`AuthoringError::UnknownAsset`] when the chosen Asset is
/// not indexed.
pub(crate) fn indexed_asset(
    world: &mut World,
    asset: &AssetAddress,
) -> Result<Indexed, AuthoringError> {
    let folder = world
        .query::<&AssetFolder>()
        .iter(world)
        .find(|folder| folder.key == asset.folder)
        .ok_or_else(|| AuthoringError::UnknownFolder(asset.folder.clone()))?;
    let indexed = folder
        .assets
        .iter()
        .find(|indexed| indexed.place == asset.place)
        .ok_or_else(|| AuthoringError::UnknownAsset {
            folder: folder.name.clone(),
            place: asset.place.clone(),
        })?;
    Ok(Indexed {
        path: folder.path.clone(),
        folder: AssetFolderReference {
            name: folder.name.clone(),
            version: folder.version.clone(),
        },
        asset: indexed.clone(),
    })
}

/// Resolves the chosen Asset for an Element placed on `layer`: finds it in its Asset Folder,
/// reads its file, and works out what the Project must record about it and its natural size.
///
/// # Errors
///
/// [`AuthoringError::NoProject`] when the Layer belongs to no Project,
/// [`AuthoringError::UnknownFolder`] or [`AuthoringError::UnknownAsset`] when the chosen Asset is
/// not indexed, or [`AuthoringError::Library`] when its file cannot be read.
pub(crate) fn resolve(
    world: &mut World,
    layer: Entity,
    asset: &AssetAddress,
) -> Result<Resolved, AuthoringError> {
    let project = project_of(world, layer)?;
    let pixels_per_cell = world.get::<Grid>(project).map_or_else(
        || Grid::default().pixels_per_cell,
        |grid| grid.pixels_per_cell,
    );
    let Indexed {
        path,
        folder,
        asset: indexed,
    } = indexed_asset(world, asset)?;
    let loaded = load_asset(&path, &indexed.place)?;

    #[expect(
        clippy::cast_precision_loss,
        reason = "cells per image are far below the range where f32 loses whole numbers"
    )]
    let size = loaded.pixel_size.as_vec2() / pixels_per_cell as f32;
    let reference = AssetReference {
        folder: folder.name.clone(),
        places: vec![indexed.place.nfc().collect()],
        name: indexed.name,
        kind: indexed.kind,
        fingerprint: loaded.fingerprint,
        byte_size: loaded.byte_size,
        pixel_size: Some(loaded.pixel_size),
    };
    Ok(Resolved {
        project,
        size,
        reference,
        folder,
    })
}

/// Places a Prop: resolves the chosen Asset and places a Prop of it on top of the Layer,
/// centred on the given position, as one history step.
///
/// # Errors
///
/// Whatever resolving the Asset reports, or [`AuthoringError::History`] when the step could not
/// be recorded.
fn place_prop(
    world: &mut World,
    layer: Entity,
    position: Vec2,
    asset: &AssetAddress,
) -> Result<(), AuthoringError> {
    let resolved = resolve(world, layer, asset)?;
    crate::record_step(
        world,
        Place {
            layer,
            position,
            resolved,
            spawned: Spawned::Prop,
            element: ElementId::new(),
        },
    )
}
