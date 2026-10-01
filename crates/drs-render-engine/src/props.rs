//! One sprite per Prop, kept in step with the model through change detection.

use bevy_asset::{AssetServer, Handle, LoadState};
use bevy_color::Color;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::Children;
use bevy_ecs::lifecycle::RemovedComponents;
use bevy_ecs::query::{Changed, Or, With};
use bevy_ecs::system::{Commands, EntityCommands, Query, Res, SystemParam};
use bevy_image::Image;
use bevy_math::Vec3;
use bevy_sprite::Sprite;
use bevy_transform::components::Transform;
use drs_library_access::asset_path;
use drs_model::{AssetFolder, AssetReferences, Element, Layer, Level, Project, Prop};
use std::collections::BTreeMap;
use unicode_normalization::UnicodeNormalization;

/// Marks a sprite entity as drawing one Element.
#[derive(Component)]
pub(crate) struct Drawing {
    /// The Element drawn.
    element: Entity,
    /// The asset path of the image shown, or `None` while a placeholder stands in.
    image: Option<String>,
}

/// Marks a sprite whose image has not finished loading.
#[derive(Component)]
pub(crate) struct Loading;

/// The flat colour that stands in for an image that could not be loaded.
const PLACEHOLDER: Color = Color::srgba(0.75, 0.3, 0.35, 0.8);

/// The parts of the model the sprites are drawn from.
#[derive(SystemParam)]
pub(crate) struct Model<'w, 's> {
    /// Each Project's Asset Reference table and its Levels.
    projects: Query<'w, 's, (&'static AssetReferences, &'static Children), With<Project>>,
    /// Each Level's Layers.
    levels: Query<'w, 's, &'static Children, With<Level>>,
    /// Each Layer's Elements in stacking order.
    layers: Query<'w, 's, &'static Children, With<Layer>>,
    /// Each Prop and what every Element has.
    elements: Query<'w, 's, (&'static Element, &'static Prop)>,
    /// The Asset Folders added on this device, which give the folder key an image loads under.
    folders: Query<'w, 's, &'static AssetFolder>,
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
    folders: Query<(), Changed<AssetFolder>>,
    mut removed_elements: RemovedComponents<Element>,
    mut removed_folders: RemovedComponents<AssetFolder>,
) -> bool {
    let removed = removed_elements.read().count() + removed_folders.read().count();
    !elements.is_empty() || !orders.is_empty() || !folders.is_empty() || removed > 0
}

/// Brings the sprites in step with the model: one per Prop at its Element's position and size,
/// stacked in the order of the Layers' children, one depth unit apart; sprites of Elements that
/// are gone are removed.
///
/// Depth counts up through every Level and Layer of every Project in the order of their
/// children, so no two Elements share a depth and a later Element is drawn over an earlier one.
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
    let mut depth: u32 = 0;
    for (references, levels) in &model.projects {
        for &level in levels {
            let Ok(layers) = model.levels.get(level) else {
                continue;
            };
            for &layer in layers {
                let Ok(elements) = model.layers.get(layer) else {
                    continue;
                };
                for &element in elements {
                    let Ok((shape, prop)) = model.elements.get(element) else {
                        continue;
                    };
                    #[expect(
                        clippy::cast_precision_loss,
                        reason = "far fewer Elements are drawn than f32 counts exactly"
                    )]
                    let z = depth as f32;
                    depth += 1;
                    let translation = Vec3::new(shape.position.x, shape.position.y, z);
                    let image = image_of(references, prop, &model.folders);
                    if let Some(sprite_entity) = unseen.remove(&element) {
                        let Ok((_, mut drawing, mut sprite, mut transform)) =
                            sprites.get_mut(sprite_entity)
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
                                image.as_deref(),
                                &asset_server,
                            );
                        }
                    } else {
                        let mut sprite = Sprite {
                            custom_size: Some(shape.size),
                            ..Sprite::default()
                        };
                        let mut spawned = commands.spawn_empty();
                        show(&mut spawned, &mut sprite, image.as_deref(), &asset_server);
                        spawned.insert((
                            sprite,
                            Transform::from_translation(translation),
                            Drawing { element, image },
                        ));
                    }
                }
            }
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
                    drawing.image.as_deref().unwrap_or_default()
                );
                placeholder(&mut sprite);
                commands.entity(entity).remove::<Loading>();
            }
        }
    }
}

/// The `lib://` path of a Prop's image: the place of the indexed Asset that one of the Asset
/// Reference's places names, under the key of the added Asset Folder with its Canonical Name.
/// `None` when no such folder is added or none of its Assets sits at a known place.
///
/// The reference's places are Unicode-normalised while the index keeps the spelling on disk, so
/// they are compared normalised and the path is built from the spelling on disk.
fn image_of(
    references: &AssetReferences,
    prop: &Prop,
    folders: &Query<&AssetFolder>,
) -> Option<String> {
    let reference = references.get(prop.asset)?;
    let folder = folders
        .iter()
        .find(|folder| folder.name == reference.folder)?;
    let place = folder
        .assets
        .iter()
        .map(|asset| &asset.place)
        .find(|place| {
            reference
                .places
                .iter()
                .any(|known| known == *place || place.nfc().eq(known.chars()))
        })?;
    Some(asset_path(&folder.key, place))
}

/// Points a sprite at an image, which starts loading, or at the placeholder when there is none.
fn show(
    entity: &mut EntityCommands,
    sprite: &mut Sprite,
    image: Option<&str>,
    asset_server: &AssetServer,
) {
    if let Some(path) = image {
        sprite.image = asset_server.load::<Image>(path.to_owned());
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
