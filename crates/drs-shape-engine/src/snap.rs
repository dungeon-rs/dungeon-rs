//! `Snap`: where a point being placed or dragged goes, on the nearest point of a Wall or a Room
//! within reach or else on the nearest corner of a Grid cell, and how far a whole Wall or Room
//! being dragged moves, in whole cells.
//!
//! Every step is a single-precision subtraction, multiplication, addition, comparison, or
//! rounding, each exact or correctly rounded, so the answer is the same on every machine.

use bevy_math::Vec2;
use drs_model::{PointOf, Pointer, Snapped, Snapping};

/// A point of a Wall or a Room that a point being snapped may go to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapTarget {
    /// Where the point is, in Grid cells.
    pub position: Vec2,
    /// Which point of which Wall or Room it is.
    pub point: PointOf,
}

/// `Snap`: where snapping puts what `pointer` snaps, given the points of the Walls and Rooms of
/// its Level in their Elements' stacking order, the bottom first; `None` while it snaps nothing.
///
/// A point goes to the nearest of `targets` no farther from the pointer than its reach, leaving
/// out the point being dragged, the later in the stacking order of two as near, and takes that
/// point's coordinates exactly; with none within reach, it goes to the nearest Grid corner, each
/// coordinate rounded to a whole number of cells, a coordinate exactly halfway rounded away from
/// zero. A move goes by the pointer's travel since the drag began, each coordinate rounded the
/// same way. The targets are scanned once, comparing squared distances.
#[must_use]
pub fn snap(pointer: &Pointer, targets: &[SnapTarget]) -> Option<Snapped> {
    match pointer.snapping {
        Snapping::Nothing => None,
        Snapping::Point { left_out } => {
            let reach = pointer.reach * pointer.reach;
            let mut nearest: Option<(f32, &SnapTarget)> = None;
            for target in targets {
                if Some(target.point) == left_out {
                    continue;
                }
                let distance = (target.position - pointer.cells).length_squared();
                // `<=` keeps the later of two as near, the one higher in the stacking order.
                if distance <= reach && nearest.is_none_or(|(best, _)| distance <= best) {
                    nearest = Some((distance, target));
                }
            }
            Some(match nearest {
                Some((_, target)) => Snapped::Point {
                    position: target.position,
                    on: Some(target.point.element),
                },
                None => Snapped::Point {
                    position: pointer.cells.round(),
                    on: None,
                },
            })
        }
        Snapping::Move { from } => Some(Snapped::Move {
            travel: (pointer.cells - from).round(),
        }),
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::missing_panics_doc,
        reason = "a test stops at the first thing that is not as expected"
    )]

    use super::*;
    use drs_model::ElementId;

    /// A Pointer at `cells` snapping a point within `reach` cells, leaving out `left_out`.
    fn placing(cells: Vec2, reach: f32, left_out: Option<PointOf>) -> Pointer {
        Pointer {
            level: None,
            cells,
            reach,
            snapping: Snapping::Point { left_out },
        }
    }

    /// The point a Pointer at `cells` within `reach` cells is snapped to among `targets`.
    fn snapped(cells: Vec2, reach: f32, targets: &[SnapTarget]) -> Option<Snapped> {
        snap(&placing(cells, reach, None), targets)
    }

    /// A target at `position`, the point `index` of `element`.
    fn target(element: ElementId, index: usize, position: Vec2) -> SnapTarget {
        SnapTarget {
            position,
            point: PointOf { element, index },
        }
    }

    /// The Grid corner a point is snapped to with nothing within reach.
    fn corner(position: Vec2) -> Snapped {
        Snapped::Point { position, on: None }
    }

    #[test]
    fn rounds_to_the_nearest_corner() {
        for (cells, expected) in [
            (Vec2::new(2.3, 4.7), Vec2::new(2.0, 5.0)),
            (Vec2::new(-2.3, -4.7), Vec2::new(-2.0, -5.0)),
            (Vec2::new(0.4, -0.4), Vec2::new(0.0, 0.0)),
            (Vec2::new(2.5, -2.5), Vec2::new(3.0, -3.0)),
            (Vec2::new(-0.5, 0.5), Vec2::new(-1.0, 1.0)),
        ] {
            assert_eq!(
                snapped(cells, 0.125, &[]),
                Some(corner(expected)),
                "{cells}"
            );
        }
    }

    #[test]
    fn the_nearest_point_within_reach_wins() {
        let (a, b) = (ElementId::new(), ElementId::new());
        let near = Vec2::new(2.3, 2.0);
        let nearer = Vec2::new(2.2, 2.0);
        let targets = [target(a, 0, near), target(b, 0, nearer)];

        let found = snapped(Vec2::new(2.05, 2.0), 0.3, &targets);

        assert_eq!(
            found,
            Some(Snapped::Point {
                position: nearer,
                on: Some(b)
            }),
            "the nearer point, though the corner at 2, 2 is nearer still"
        );
        assert_eq!(
            snapped(Vec2::new(1.8, 2.0), 0.3, &targets),
            Some(corner(Vec2::new(2.0, 2.0))),
            "both points beyond reach"
        );
    }

    #[test]
    fn a_tie_goes_to_the_later_point() {
        let (below, above) = (ElementId::new(), ElementId::new());
        let at = Vec2::new(3.25, 1.75);
        let targets = [target(below, 2, at), target(above, 0, at)];

        assert_eq!(
            snapped(Vec2::new(3.3, 1.8), 0.25, &targets),
            Some(Snapped::Point {
                position: at,
                on: Some(above)
            })
        );
    }

    #[test]
    fn the_left_out_point_is_skipped() {
        let room = ElementId::new();
        let dragged = PointOf {
            element: room,
            index: 1,
        };
        let neighbour = Vec2::new(4.1, 0.2);
        let targets = [
            target(room, 0, neighbour),
            SnapTarget {
                position: Vec2::new(4.3, 0.4),
                point: dragged,
            },
        ];

        let pointer = placing(Vec2::new(4.4, 0.45), 0.25, Some(dragged));
        assert_eq!(snap(&pointer, &targets), Some(corner(Vec2::new(4.0, 0.0))));
        let pointer = placing(Vec2::new(4.25, 0.35), 0.25, Some(dragged));
        assert_eq!(
            snap(&pointer, &targets),
            Some(Snapped::Point {
                position: neighbour,
                on: Some(room)
            })
        );
    }

    #[test]
    fn a_move_rounds_its_travel() {
        let from = Vec2::new(0.75, -0.25);
        for (cells, travel) in [
            (Vec2::new(2.875, 1.0), Vec2::new(2.0, 1.0)),
            (Vec2::new(-1.5, -2.0), Vec2::new(-2.0, -2.0)),
            (Vec2::new(1.25, 0.25), Vec2::new(1.0, 1.0)),
            (Vec2::new(0.25, -0.75), Vec2::new(-1.0, -1.0)),
        ] {
            let pointer = Pointer {
                level: None,
                cells,
                reach: 0.125,
                snapping: Snapping::Move { from },
            };
            assert_eq!(
                snap(&pointer, &[]),
                Some(Snapped::Move { travel }),
                "{cells}"
            );
        }
        let still = Pointer {
            snapping: Snapping::Nothing,
            ..Pointer::default()
        };
        assert_eq!(snap(&still, &[]), None);
    }
}
