//! One mesh with a flat-colour Material per Element drawn as a stroked path, kept in step with the
//! model through change detection.

use crate::drawn_as;
use crate::stacking::Stacking;
use bevy_asset::{Assets, Handle, RenderAssetUsages};
use bevy_color::Color;
use bevy_ecs::change_detection::DetectChanges;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::Children;
use bevy_ecs::lifecycle::RemovedComponents;
use bevy_ecs::query::{Changed, Or, With};
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{Commands, Query, Res, ResMut, SystemParam};
use bevy_ecs::world::Ref;
use bevy_math::Vec3;
use bevy_mesh::{Indices, Mesh, Mesh2d, PrimitiveTopology};
use bevy_sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d};
use bevy_transform::components::Transform;
use drs_model::{
    Colour, DrawnAs, Element, ElementKindRegistry, Layer, Level, Project, Wall, WallShape,
};
use std::collections::{BTreeMap, BTreeSet};

/// Marks a mesh entity as drawing one Wall.
#[derive(Component)]
pub(crate) struct WallDrawing {
    /// The Wall drawn.
    element: Entity,
    /// The colour its Material has.
    colour: Colour,
}

/// Whether anything the Walls' meshes depend on changed since they were last brought in step.
#[expect(
    clippy::type_complexity,
    reason = "a Bevy query filter is spelled out by the components it watches"
)]
pub(crate) fn walls_changed(
    walls: Query<(), Or<(Changed<Wall>, Changed<WallShape>)>>,
    orders: Query<
        (),
        (
            Changed<Children>,
            Or<(With<Project>, With<Level>, With<Layer>)>,
        ),
    >,
    mut removed: RemovedComponents<WallShape>,
) -> bool {
    let removed = removed.read().count();
    !walls.is_empty() || !orders.is_empty() || removed > 0
}

/// One flat-colour Material per colour a Wall is drawn in, shared by every Wall of that colour.
#[derive(Resource, Default)]
pub(crate) struct WallMaterials(BTreeMap<[u8; 3], Handle<ColorMaterial>>);

impl WallMaterials {
    /// The Material of `colour`, added the first time a Wall is drawn in it.
    ///
    /// It blends rather than being opaque, though the colour is, so that the Wall sorts with the
    /// sprites by depth.
    fn of(
        &mut self,
        colour: Colour,
        materials: &mut Assets<ColorMaterial>,
    ) -> Handle<ColorMaterial> {
        self.0
            .entry(key(colour))
            .or_insert_with(|| {
                materials.add(ColorMaterial {
                    color: colour_of(colour),
                    alpha_mode: AlphaMode2d::Blend,
                    ..ColorMaterial::default()
                })
            })
            .clone()
    }
}

/// The assets the Walls are drawn with.
#[derive(SystemParam)]
pub(crate) struct WallAssets<'w> {
    /// The meshes, one per Wall.
    meshes: ResMut<'w, Assets<Mesh>>,
    /// The Materials, one per colour.
    materials: ResMut<'w, Assets<ColorMaterial>>,
    /// Which Material each colour has.
    shared: ResMut<'w, WallMaterials>,
}

/// A colour as the shared Materials are kept by.
fn key(colour: Colour) -> [u8; 3] {
    [colour.red, colour.green, colour.blue]
}

/// The Material colour of a Wall's colour.
fn colour_of(colour: Colour) -> Color {
    Color::srgb_u8(colour.red, colour.green, colour.blue)
}

/// The mesh of a Wall's stroke: its vertices in cells, its triangles, and the arc length of the
/// line at each vertex as the first texture coordinate, for a Material that repeats along it.
fn mesh_of(shape: &WallShape) -> Mesh {
    let positions: Vec<[f32; 3]> = shape
        .mesh
        .vertices
        .iter()
        .map(|vertex| [vertex.x, vertex.y, 0.0])
        .collect();
    let along: Vec<[f32; 2]> = shape
        .mesh
        .arc_lengths
        .iter()
        .map(|length| [*length, 0.0])
        .collect();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, along)
    .with_inserted_indices(Indices::U32(shape.mesh.indices.clone()))
}

/// Brings the Walls' meshes in step with the model: one mesh per Element whose kind is drawn as a
/// stroked path and that has its derived shape, in its colour, at its depth in the stacking order
/// shared with the sprites, replaced when the shape changes; the meshes of Walls that are gone
/// are removed. A Wall whose shape is not derived yet is not drawn that frame.
///
/// Walls of one colour share its Material.
pub(crate) fn sync_walls(
    mut commands: Commands,
    stacking: Stacking,
    kinds: Option<Res<ElementKindRegistry>>,
    walls: Query<(&Element, &Wall, Ref<WallShape>)>,
    mut drawings: Query<(
        Entity,
        &mut WallDrawing,
        &Mesh2d,
        &mut MeshMaterial2d<ColorMaterial>,
        &mut Transform,
    )>,
    mut assets: WallAssets,
) {
    let mut unseen: BTreeMap<Entity, Entity> = drawings
        .iter()
        .map(|(drawing, wall, ..)| (wall.element, drawing))
        .collect();
    let mut in_use = BTreeSet::new();
    for stacked in stacking.in_order() {
        let Ok((element, wall, shape)) = walls.get(stacked.element) else {
            continue;
        };
        match drawn_as(kinds.as_deref(), element) {
            Some(DrawnAs::StrokedPath) => {}
            Some(DrawnAs::Image | DrawnAs::PaintedSurface | DrawnAs::FilledOutline) | None => {
                continue;
            }
        }
        in_use.insert(key(wall.colour));
        let translation = Vec3::new(0.0, 0.0, stacked.depth);
        if let Some(drawn) = unseen.remove(&stacked.element) {
            let Ok((_, mut drawing, mesh, mut material, mut transform)) = drawings.get_mut(drawn)
            else {
                continue;
            };
            if transform.translation != translation {
                transform.translation = translation;
            }
            if shape.is_changed() && assets.meshes.insert(&mesh.0, mesh_of(&shape)).is_err() {
                log::warn!("the mesh of a Wall could not be replaced");
            }
            if drawing.colour != wall.colour {
                drawing.colour = wall.colour;
                material.0 = assets.shared.of(wall.colour, &mut assets.materials);
            }
        } else {
            commands.spawn((
                Mesh2d(assets.meshes.add(mesh_of(&shape))),
                MeshMaterial2d(assets.shared.of(wall.colour, &mut assets.materials)),
                Transform::from_translation(translation),
                WallDrawing {
                    element: stacked.element,
                    colour: wall.colour,
                },
            ));
        }
    }
    for drawn in unseen.into_values() {
        commands.entity(drawn).despawn();
    }
    // A colour no Wall is drawn in any more lets its Material go.
    assets.shared.0.retain(|colour, _| in_use.contains(colour));
}
