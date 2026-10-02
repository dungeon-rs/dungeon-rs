//! One sprite per Element drawn as an image, or of a kind this editor does not know, kept in step
//! with the model through change detection: a Prop upright, a Portal turned and mirrored.

use crate::drawn_as;
use crate::stacking::Stacking;
use bevy_asset::{AssetPath, AssetServer, Handle, LoadState};
use bevy_color::Color;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::Children;
use bevy_ecs::lifecycle::RemovedComponents;
use bevy_ecs::query::{Changed, Or, With, Without};
use bevy_ecs::system::{Commands, EntityCommands, Query, Res, SystemParam};
use bevy_image::Image;
use bevy_math::{Quat, Vec3, ops};
use bevy_sprite::Sprite;
use bevy_transform::components::Transform;
use drs_library_access::asset_path;
use drs_model::{
    AssetReferenceRow, DrawnAs, Element, ElementKindRegistry, Layer, Level, Portal, Project, Prop,
    Resolution, ResolutionTable, RoomShape, ShownAsset, WallShape,
};
use std::collections::BTreeMap;

/// Marks a sprite entity as drawing one Element.
#[derive(Component)]
pub(crate) struct Drawing {
    /// The Element drawn.
    element: Entity,
    /// The asset path of the image shown, or `None` while a placeholder stands in.
    image: Option<AssetPath<'static>>,
}

/// Marks a sprite whose image has not finished loading.
#[derive(Component)]
pub(crate) struct Loading;

/// The flat colour that stands in for an image that could not be loaded, a Missing Asset, or an
/// Element of a kind this editor does not know.
const PLACEHOLDER: Color = Color::srgba(0.75, 0.3, 0.35, 0.8);

/// The parts of the model the sprites are drawn from.
#[derive(SystemParam)]
pub(crate) struct Model<'w, 's> {
    /// The stacking order.
    stacking: Stacking<'w, 's>,
    /// Each Project's resolution table.
    projects: Query<'w, 's, &'static ResolutionTable, With<Project>>,
    /// What every Element has, the Asset it shows, and the Portal it is when it is one.
    elements: Query<'w, 's, (&'static Element, ShownAsset, Option<&'static Portal>)>,
    /// How each known kind is drawn.
    kinds: Option<Res<'w, ElementKindRegistry>>,
}

/// Whether anything the sprites depend on changed since the sprites were last brought in step.
///
/// An Element drawn from its derived stroke is left out: it has no sprite, and its box changes
/// with every move of its points.
#[expect(
    clippy::type_complexity,
    reason = "a Bevy query filter is spelled out by the components it watches"
)]
pub(crate) fn props_changed(
    elements: Query<
        (),
        (
            Or<(Changed<Element>, Changed<Prop>, Changed<Portal>)>,
            Without<WallShape>,
            Without<RoomShape>,
        ),
    >,
    orders: Query<
        (),
        (
            Changed<Children>,
            Or<(With<Project>, With<Level>, With<Layer>)>,
        ),
    >,
    tables: Query<(), Changed<ResolutionTable>>,
    mut removed_elements: RemovedComponents<Element>,
) -> bool {
    let removed = removed_elements.read().count();
    !elements.is_empty() || !orders.is_empty() || !tables.is_empty() || removed > 0
}

/// Brings the sprites in step with the model: one per Element whose kind is drawn as an image or
/// is not known, at its position and size and at its depth in the stacking order, a Portal
/// turned by its rotation and flipped across its length when mirrored; sprites of Elements that
/// are gone are removed. A Prop or a Portal shows the image its Asset Reference resolves to; a
/// Missing Asset and an Element of a kind this editor does not know show the placeholder, turned
/// and flipped as the image would be.
pub(crate) fn sync_props(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    model: Model,
    mut sprites: Query<(Entity, &mut Drawing, &mut Sprite, &mut Transform)>,
) {
    let mut unseen: BTreeMap<Entity, Entity> = sprites
        .iter()
        .map(|(sprite, drawing, _, _)| (drawing.element, sprite))
        .collect();
    for stacked in model.stacking.in_order() {
        let element = stacked.element;
        let (Ok(resolutions), Ok((shape, shown, portal))) = (
            model.projects.get(stacked.project),
            model.elements.get(element),
        ) else {
            continue;
        };
        match drawn_as(model.kinds.as_deref(), shape) {
            Some(DrawnAs::StrokedPath | DrawnAs::PaintedSurface | DrawnAs::FilledOutline) => {
                continue;
            }
            Some(DrawnAs::Image) | None => {}
        }
        let translation = Vec3::new(shape.position.x, shape.position.y, stacked.depth);
        let rotation = portal.map_or(Quat::IDENTITY, |portal| turn(portal.rotation));
        let mirrored = portal.is_some_and(|portal| portal.mirrored);
        let image = image_of(resolutions, shown.row());
        if let Some(sprite_entity) = unseen.remove(&element) {
            let Ok((_, mut drawing, mut sprite, mut transform)) = sprites.get_mut(sprite_entity)
            else {
                continue;
            };
            if transform.translation != translation {
                transform.translation = translation;
            }
            if transform.rotation != rotation {
                transform.rotation = rotation;
            }
            if sprite.custom_size != Some(shape.size) {
                sprite.custom_size = Some(shape.size);
            }
            if sprite.flip_y != mirrored {
                sprite.flip_y = mirrored;
            }
            if drawing.image != image {
                drawing.image.clone_from(&image);
                show(
                    &mut commands.entity(sprite_entity),
                    &mut sprite,
                    image.as_ref(),
                    &asset_server,
                );
            }
        } else {
            let mut sprite = Sprite {
                custom_size: Some(shape.size),
                flip_y: mirrored,
                ..Sprite::default()
            };
            let mut spawned = commands.spawn_empty();
            show(&mut spawned, &mut sprite, image.as_ref(), &asset_server);
            spawned.insert((
                sprite,
                Transform::from_translation(translation).with_rotation(rotation),
                Drawing { element, image },
            ));
        }
    }
    for sprite in unseen.into_values() {
        commands.entity(sprite).despawn();
    }
}

/// Settles the sprites whose image was loading: a loaded image stays, a failed one gives way to
/// a placeholder of the same size, and the editor keeps running either way.
pub(crate) fn settle_loads(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut sprites: Query<(Entity, &Drawing, &mut Sprite), With<Loading>>,
) {
    for (entity, drawing, mut sprite) in &mut sprites {
        match asset_server.get_load_state(sprite.image.id()) {
            None | Some(LoadState::NotLoaded | LoadState::Loading) => {}
            Some(LoadState::Loaded) => {
                commands.entity(entity).remove::<Loading>();
            }
            Some(LoadState::Failed(error)) => {
                log::warn!(
                    "the image {} could not be loaded, so a placeholder stands in: {error}",
                    drawing
                        .image
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_default()
                );
                placeholder(&mut sprite);
                commands.entity(entity).remove::<Loading>();
            }
        }
    }
}

/// A turn counter-clockwise by `angle` radians about the axis out of the Level, built through
/// the deterministic maths functions so the Export is the same on every machine.
fn turn(angle: f32) -> Quat {
    let (sine, cosine) = ops::sin_cos(angle / 2.0);
    Quat::from_xyzw(0.0, 0.0, sine, cosine)
}

/// The `lib://` path of an Element's image: where the resolution table says the Asset Reference
/// in its row loads from on this device. `None` for a Missing Asset, for a row not yet resolved,
/// and for an Element that shows no Asset.
fn image_of(
    resolutions: &ResolutionTable,
    row: Option<AssetReferenceRow>,
) -> Option<AssetPath<'static>> {
    match resolutions.get(row?)? {
        Resolution::Resolved { folder, place } => Some(asset_path(folder, place)),
        Resolution::Missing(_) => None,
    }
}

/// Points a sprite at an image, which starts loading, or at the placeholder when there is none.
fn show(
    entity: &mut EntityCommands,
    sprite: &mut Sprite,
    image: Option<&AssetPath<'static>>,
    asset_server: &AssetServer,
) {
    if let Some(path) = image {
        sprite.image = asset_server.load::<Image>(path);
        sprite.color = Color::WHITE;
        entity.insert(Loading);
    } else {
        placeholder(sprite);
        entity.remove::<Loading>();
    }
}

/// Makes a sprite the flat coloured placeholder, keeping its size.
fn placeholder(sprite: &mut Sprite) {
    sprite.image = Handle::default();
    sprite.color = PLACEHOLDER;
}
