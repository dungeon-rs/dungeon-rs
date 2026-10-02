//! Meshes with flat-colour Materials for the Elements drawn from a derived outline, kept in step
//! with the model through change detection: one per Element drawn as a stroked path, a Wall, and
//! two per Element drawn as a filled outline, a Room, its floor and its Walls.

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
    Colour, DrawnAs, Element, ElementKindRegistry, FillMesh, Layer, Level, Project, Room,
    RoomShape, StrokeMesh, Wall, WallShape,
};
use std::collections::{BTreeMap, BTreeSet};

/// Which part of an Element a mesh draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Part {
    /// The stroke of a Wall's line or of a Room's Walls.
    Stroke,
    /// A Room's floor.
    Floor,
}

/// How far below its Element's depth a floor is drawn: under the Element's own Walls and over
/// the Element before it, one whole unit below.
const FLOOR_BELOW: f32 = 0.5;

/// Marks a mesh entity as drawing one part of one Element.
#[derive(Component)]
pub(crate) struct OutlineDrawing {
    /// The Element drawn.
    element: Entity,
    /// The part of it drawn.
    part: Part,
    /// The colour its Material has.
    colour: Colour,
}

/// Whether anything the meshes of Walls and Rooms depend on changed since they were last brought
/// in step.
#[expect(
    clippy::type_complexity,
    reason = "a Bevy query filter is spelled out by the components it watches"
)]
pub(crate) fn walls_changed(
    walls: Query<(), Or<(Changed<Wall>, Changed<WallShape>)>>,
    rooms: Query<(), Or<(Changed<Room>, Changed<RoomShape>)>>,
    orders: Query<
        (),
        (
            Changed<Children>,
            Or<(With<Project>, With<Level>, With<Layer>)>,
        ),
    >,
    mut removed: RemovedComponents<WallShape>,
    mut removed_rooms: RemovedComponents<RoomShape>,
) -> bool {
    let removed = removed.read().count() + removed_rooms.read().count();
    !walls.is_empty() || !rooms.is_empty() || !orders.is_empty() || removed > 0
}

/// One flat-colour Material per colour a Wall, a Room's Walls, or a Room's floor is drawn in,
/// shared by everything drawn in that colour.
#[derive(Resource, Default)]
pub(crate) struct OutlineMaterials(BTreeMap<[u8; 3], Handle<ColorMaterial>>);

impl OutlineMaterials {
    /// The Material of `colour`, added the first time something is drawn in it.
    ///
    /// It blends rather than being opaque, though the colour is, so that the mesh sorts with the
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

/// The assets the Walls and Rooms are drawn with.
#[derive(SystemParam)]
pub(crate) struct OutlineAssets<'w> {
    /// The meshes, one per Wall and two per Room.
    meshes: ResMut<'w, Assets<Mesh>>,
    /// The Materials, one per colour.
    materials: ResMut<'w, Assets<ColorMaterial>>,
    /// Which Material each colour has.
    shared: ResMut<'w, OutlineMaterials>,
}

/// A colour as the shared Materials are kept by.
fn key(colour: Colour) -> [u8; 3] {
    [colour.red, colour.green, colour.blue]
}

/// The Material colour of a Wall's colour.
fn colour_of(colour: Colour) -> Color {
    Color::srgb_u8(colour.red, colour.green, colour.blue)
}

/// The mesh of a stroke: its vertices in cells, its triangles, and the arc length of the line at
/// each vertex as the first texture coordinate, for a Material that repeats along it.
fn stroke_mesh(stroke: &StrokeMesh) -> Mesh {
    let positions: Vec<[f32; 3]> = stroke
        .vertices
        .iter()
        .map(|vertex| [vertex.x, vertex.y, 0.0])
        .collect();
    let along: Vec<[f32; 2]> = stroke
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
    .with_inserted_indices(Indices::U32(stroke.indices.clone()))
}

/// The mesh of a floor: its vertices in cells, with their position as the first texture
/// coordinate, for a Material that repeats across it, and its triangles.
fn floor_mesh(floor: &FillMesh) -> Mesh {
    let positions: Vec<[f32; 3]> = floor
        .vertices
        .iter()
        .map(|vertex| [vertex.x, vertex.y, 0.0])
        .collect();
    let across: Vec<[f32; 2]> = floor
        .vertices
        .iter()
        .map(|vertex| [vertex.x, vertex.y])
        .collect();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, across)
    .with_inserted_indices(Indices::U32(floor.indices.clone()))
}

/// One part of an Element to draw: which, in what colour, at what depth, whether its shape
/// changed since it was last drawn, and how its mesh is built.
struct Piece<'a> {
    /// The Element.
    element: Entity,
    /// The part.
    part: Part,
    /// Its colour.
    colour: Colour,
    /// Its depth.
    depth: f32,
    /// Whether the derived shape it is built from changed.
    changed: bool,
    /// Builds its mesh.
    mesh: Box<dyn Fn() -> Mesh + 'a>,
}

/// The meshes being brought in step: the drawings of the last frame not yet seen this frame, and
/// the colours drawn in.
struct Sync {
    /// The drawings not yet seen, by Element and part.
    unseen: BTreeMap<(Entity, Part), Entity>,
    /// The colours something is drawn in this frame.
    in_use: BTreeSet<[u8; 3]>,
}

/// Brings the meshes of Walls and Rooms in step with the model: one mesh per Element whose kind
/// is drawn as a stroked path, and a floor mesh under a stroke mesh per Element whose kind is
/// drawn as a filled outline, once it has its derived shape, each in its colour, at the
/// Element's depth in the stacking order shared with the sprites, the floor half a unit below,
/// replaced when the shape changes; the meshes of Elements that are gone are removed. An Element
/// whose shape is not derived yet is not drawn that frame.
///
/// Everything drawn in one colour shares its Material.
pub(crate) fn sync_walls(
    mut commands: Commands,
    stacking: Stacking,
    kinds: Option<Res<ElementKindRegistry>>,
    walls: Query<(&Element, &Wall, Ref<WallShape>)>,
    rooms: Query<(&Element, &Room, Ref<RoomShape>)>,
    mut drawings: Query<(
        Entity,
        &mut OutlineDrawing,
        &Mesh2d,
        &mut MeshMaterial2d<ColorMaterial>,
        &mut Transform,
    )>,
    mut assets: OutlineAssets,
) {
    let mut sync = Sync {
        unseen: drawings
            .iter()
            .map(|(drawing, drawn, ..)| ((drawn.element, drawn.part), drawing))
            .collect(),
        in_use: BTreeSet::new(),
    };
    for stacked in stacking.in_order() {
        if let Ok((element, wall, shape)) = walls.get(stacked.element) {
            match drawn_as(kinds.as_deref(), element) {
                Some(DrawnAs::StrokedPath) => {
                    let piece = Piece {
                        element: stacked.element,
                        part: Part::Stroke,
                        colour: wall.colour,
                        depth: stacked.depth,
                        changed: shape.is_changed(),
                        mesh: Box::new(|| stroke_mesh(&shape.mesh)),
                    };
                    draw(&mut sync, piece, &mut commands, &mut drawings, &mut assets);
                }
                Some(DrawnAs::Image | DrawnAs::PaintedSurface | DrawnAs::FilledOutline) | None => {}
            }
        } else if let Ok((element, room, shape)) = rooms.get(stacked.element) {
            match drawn_as(kinds.as_deref(), element) {
                Some(DrawnAs::FilledOutline) => {
                    let changed = shape.is_changed();
                    let floor = Piece {
                        element: stacked.element,
                        part: Part::Floor,
                        colour: room.floor_colour,
                        depth: stacked.depth - FLOOR_BELOW,
                        changed,
                        mesh: Box::new(|| floor_mesh(&shape.floor)),
                    };
                    draw(&mut sync, floor, &mut commands, &mut drawings, &mut assets);
                    let walls = Piece {
                        element: stacked.element,
                        part: Part::Stroke,
                        colour: room.wall_colour,
                        depth: stacked.depth,
                        changed,
                        mesh: Box::new(|| stroke_mesh(&shape.walls.mesh)),
                    };
                    draw(&mut sync, walls, &mut commands, &mut drawings, &mut assets);
                }
                Some(DrawnAs::Image | DrawnAs::StrokedPath | DrawnAs::PaintedSurface) | None => {}
            }
        }
    }
    for drawn in sync.unseen.into_values() {
        commands.entity(drawn).despawn();
    }
    // A colour nothing is drawn in any more lets its Material go.
    let in_use = sync.in_use;
    assets.shared.0.retain(|colour, _| in_use.contains(colour));
}

/// Draws one piece: updates the mesh entity that drew it last frame, or spawns one.
fn draw(
    sync: &mut Sync,
    piece: Piece,
    commands: &mut Commands,
    drawings: &mut Query<(
        Entity,
        &mut OutlineDrawing,
        &Mesh2d,
        &mut MeshMaterial2d<ColorMaterial>,
        &mut Transform,
    )>,
    assets: &mut OutlineAssets,
) {
    sync.in_use.insert(key(piece.colour));
    let translation = Vec3::new(0.0, 0.0, piece.depth);
    if let Some(drawn) = sync.unseen.remove(&(piece.element, piece.part)) {
        let Ok((_, mut drawing, mesh, mut material, mut transform)) = drawings.get_mut(drawn)
        else {
            return;
        };
        if transform.translation != translation {
            transform.translation = translation;
        }
        if piece.changed && assets.meshes.insert(&mesh.0, (piece.mesh)()).is_err() {
            log::warn!("the mesh of a Wall or a Room could not be replaced");
        }
        if drawing.colour != piece.colour {
            drawing.colour = piece.colour;
            material.0 = assets.shared.of(piece.colour, &mut assets.materials);
        }
    } else {
        commands.spawn((
            Mesh2d(assets.meshes.add((piece.mesh)())),
            MeshMaterial2d(assets.shared.of(piece.colour, &mut assets.materials)),
            Transform::from_translation(translation),
            OutlineDrawing {
                element: piece.element,
                part: piece.part,
                colour: piece.colour,
            },
        ));
    }
}
