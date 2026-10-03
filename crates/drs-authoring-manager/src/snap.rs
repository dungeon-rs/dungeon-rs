//! The snapped point: where the shape Engine's Snap puts what the Pointer snaps, among the points
//! of the Walls and Rooms on the Pointer's Level.

use bevy_ecs::change_detection::DetectChanges;
use bevy_ecs::hierarchy::Children;
use bevy_ecs::lifecycle::RemovedComponents;
use bevy_ecs::query::{Changed, Or, With};
use bevy_ecs::system::{Query, Res, ResMut};
use drs_model::{ElementId, Layer, Level, PointOf, Pointer, Room, SnappedPoint, Snapping, Wall};
use drs_shape_engine::{SnapTarget, snap};

/// The Walls and Rooms as snapping reads their points.
type Outlines<'w, 's> = Query<
    'w,
    's,
    (
        &'static ElementId,
        Option<&'static Wall>,
        Option<&'static Room>,
    ),
>;

/// Derives the snapped point whenever the Pointer changed or a Wall or Room was placed, edited,
/// or removed: Snap is given, for a point, the points of every Wall and Room on the Pointer's
/// Level, its Layers and their Elements in stacking order, the bottom first, and for a move or
/// nothing, no point at all, so the Level is read only when a point snaps. The snapped point is written only
/// when its answer differs, or when it answers another Pointer apart from where the pointer is,
/// so a reader's change detection means a real change.
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
    outlines: Outlines,
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
                .map(|level| targets(&levels, &layers, &outlines, level))
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
/// first; the points of a Wall or a Room in their own order.
fn targets(
    levels: &Query<&Children, With<Level>>,
    layers: &Query<&Children, With<Layer>>,
    outlines: &Outlines,
    level: bevy_ecs::entity::Entity,
) -> Vec<SnapTarget> {
    let Ok(level) = levels.get(level) else {
        return Vec::new();
    };
    level
        .iter()
        .filter_map(|layer| layers.get(*layer).ok())
        .flat_map(|elements| elements.iter())
        .filter_map(|element| outlines.get(*element).ok())
        .flat_map(|(id, wall, room)| {
            let points = wall
                .map(|wall| wall.points.as_slice())
                .or_else(|| room.map(|room| room.points.as_slice()))
                .unwrap_or_default();
            points
                .iter()
                .enumerate()
                .map(|(index, position)| SnapTarget {
                    position: *position,
                    point: PointOf {
                        element: *id,
                        index,
                    },
                })
        })
        .collect()
}
