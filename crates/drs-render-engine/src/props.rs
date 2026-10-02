//! One sprite per Element but a Wall, kept in step with the model through change detection.

use crate::stacking::Stacking;
use bevy_asset::{AssetPath, AssetServer, Handle, LoadState};
use bevy_color::Color;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::Children;
use bevy_ecs::lifecycle::RemovedComponents;
use bevy_ecs::query::{Changed, Has, Or, With};
use bevy_ecs::system::{Commands, EntityCommands, Query, Res, SystemParam};
use bevy_image::Image;
use bevy_math::Vec3;
use bevy_sprite::Sprite;
use bevy_transform::components::Transform;
use drs_library_access::asset_path;
use drs_model::{Element, Layer, Level, Project, Prop, Resolution, ResolutionTable, Wall};
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
    /// What every Element has, the Prop it is when it is one, and whether it is a Wall, which is
    /// drawn as a stroke rather than a sprite.
    elements: Query<'w, 's, (&'static Element, Option<&'static Prop>, Has<Wall>)>,
}

/// Whether anything the sprites depend on changed since the sprites were last brought in step.
#[expect(
    clippy::type_complexity,
    reason = "a Bevy query filter is spelled out by the components it watches"
)]
pub(crate) fn props_changed(
    elements: Query<(), Or<(Changed<Element>, Changed<Prop>)>>,
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

/// Brings the sprites in step with the model: one per Element but a Wall, at its position and
/// size and at its depth in the stacking order; sprites of Elements that are gone are removed. A
/// Prop shows the image its Asset Reference resolves to; a Missing Asset and an Element of a kind
/// this editor does not know show the placeholder.
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
        let (Ok(resolutions), Ok((shape, prop, wall))) = (
            model.projects.get(stacked.project),
            model.elements.get(element),
        ) else {
            continue;
        };
        if wall {
            continue;
        }
        let translation = Vec3::new(shape.position.x, shape.position.y, stacked.depth);
        let image = image_of(resolutions, prop);
        if let Some(sprite_entity) = unseen.remove(&element) {
            let Ok((_, mut drawing, mut sprite, mut transform)) = sprites.get_mut(sprite_entity)
            else {
                continue;
            };
            if transform.translation != translation {
                transform.translation = translation;
            }
            if sprite.custom_size != Some(shape.size) {
                sprite.custom_size = Some(shape.size);
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
                ..Sprite::default()
            };
            let mut spawned = commands.spawn_empty();
            show(&mut spawned, &mut sprite, image.as_ref(), &asset_server);
            spawned.insert((
                sprite,
                Transform::from_translation(translation),
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

/// The `lib://` path of an Element's image: where the resolution table says its Prop's Asset
/// Reference loads from on this device. `None` for a Missing Asset, for a row not yet resolved,
/// and for an Element that is no Prop.
fn image_of(resolutions: &ResolutionTable, prop: Option<&Prop>) -> Option<AssetPath<'static>> {
    match resolutions.get(prop?.asset)? {
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
