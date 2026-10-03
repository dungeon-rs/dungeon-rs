//! The snapped point: where the shape Engine's Snap puts what the Pointer snaps, among the points
//! of the Walls and Rooms on the Pointer's Level.

use crate::outline::OutlineHost;
use bevy_ecs::change_detection::DetectChanges;
use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::Children;
use bevy_ecs::lifecycle::RemovedComponents;
use bevy_ecs::query::{Changed, Or, With};
use bevy_ecs::system::{Query, Res, ResMut};
use bevy_math::Vec2;
use drs_model::{ElementId, Layer, Level, PointOf, Pointer, Room, SnappedPoint, Snapping, Wall};
use drs_shape_engine::{SnapTarget, snap};

/// The Walls or the Rooms as snapping reads their points.
type SnapSources<'w, 's, H> = Query<'w, 's, (&'static ElementId, &'static H)>;

/// Derives the snapped point whenever the Pointer changed or a Wall or Room was placed, edited,
/// or removed: Snap is given, for a point, the points of every Wall and Room on the Pointer's
/// Level, its Layers and their Elements in stacking order, the bottom first, and for a move or
/// nothing no point at all, so the Level is read only when a point snaps. The snapped point is
/// written only when its answer differs, or when it answers another Pointer apart from where the
/// pointer is, so a reader's change detection means a real change.
#[expect(
    clippy::too_many_arguments,
    clippy::type_complexity,
    reason = "a Bevy system is spelled out by the components it reads and writes"
)]
pub(crate) fn derive_snapped_point(
    pointer: Res<Pointer>,
    mut snapped: ResMut<SnappedPoint>,
    changed: Query<(), Or<(Changed<Wall>, Changed<Room>)>>,
    mut removed_walls: RemovedComponents<Wall>,
    mut removed_rooms: RemovedComponents<Room>,
    levels: Query<&Children, With<Level>>,
    layers: Query<&Children, With<Layer>>,
    walls: SnapSources<Wall>,
    rooms: SnapSources<Room>,
) {
    let removed = removed_walls.read().count() + removed_rooms.read().count() > 0;
    if !pointer.is_changed() && changed.is_empty() && !removed {
        return;
    }
    // Only a point goes to other points: nothing and a move need none read.
    let answer = match pointer.snapping {
        Snapping::Nothing => None,
        Snapping::Move { .. } => snap(&pointer, &[]),
        Snapping::Point { .. } => {
            let targets = pointer
                .level
                .map(|level| {
                    targets(&levels, &layers, level, |element| {
                        points_of(&walls, element).or_else(|| points_of(&rooms, element))
                    })
                })
                .unwrap_or_default();
            snap(&pointer, &targets)
        }
    };
    if answer != snapped.snapped || !snapped.pointer.same_but_for_position(&pointer) {
        *snapped = SnappedPoint {
            snapped: answer,
            pointer: *pointer,
        };
    }
}

/// The points of every Wall and Room on the Layers of `level`, in stacking order, the bottom
/// first, as `points_of` reads each Element's; the points of a Wall or a Room in their own order.
fn targets(
    levels: &Query<&Children, With<Level>>,
    layers: &Query<&Children, With<Layer>>,
    level: Entity,
    points_of: impl Fn(Entity) -> Option<(ElementId, Vec<Vec2>)>,
) -> Vec<SnapTarget> {
    let Ok(level) = levels.get(level) else {
        return Vec::new();
    };
    level
        .iter()
        .filter_map(|layer| layers.get(*layer).ok())
        .flat_map(|elements| elements.iter())
        .filter_map(|element| points_of(*element))
        .flat_map(|(id, points)| {
            points
                .into_iter()
                .enumerate()
                .map(move |(index, position)| SnapTarget {
                    position,
                    point: PointOf { element: id, index },
                })
        })
        .collect()
}

/// The identity and the points of `entity` when it is an `H`, read through its outline.
fn points_of<H: OutlineHost>(
    sources: &SnapSources<H>,
    entity: Entity,
) -> Option<(ElementId, Vec<Vec2>)> {
    let (id, outline) = sources.get(entity).ok()?;
    Some((*id, outline.path().points))
}
