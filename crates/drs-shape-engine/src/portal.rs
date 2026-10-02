//! `AnchorPortals`: where each Portal set into a Wall or a Room's Walls stands and the stretch of
//! the line it covers, and where each anchor goes when a point is added or removed. A Room's
//! outline is closed, so lengths along it wrap past its first point.
//!
//! A Portal is anchored by a segment and a parameter along it, never by arc length, so a point
//! dragged elsewhere on the Wall never moves it. The centre and the direction are evaluated on
//! the exact curve; lengths along the line are measured over the flattened line, summing its
//! chords with square roots and arithmetic alone, so they come out the same on every machine.

use crate::path::{Curve, Path, flatten};
use crate::wall::{Measured, length, vector};
use bevy_math::{Vec2, ops};
use drs_model::{LinePlace, LinePoint, Stretch};
use kurbo::{ParamCurve, ParamCurveDeriv};

/// Shorter than this, in Grid cells per unit of parameter, a segment has no direction.
const NO_DIRECTION: f64 = 1e-9;

/// A Portal set into a Wall or a Room's Walls as `AnchorPortals` sees it: where along the
/// outline it is set, and how wide it is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortalSetting {
    /// The segment of a Wall or the edge of a Room, counted from zero.
    pub segment: usize,
    /// The parameter along it, from zero to one.
    pub t: f32,
    /// The Portal's width in Grid cells.
    pub width: f32,
}

/// Where a Portal set into a Wall or a Room's Walls stands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Standing {
    /// Its centre: the point of its segment or edge at its parameter.
    pub centre: Vec2,
    /// The angle of the outline's direction there, in radians counter-clockwise from the x
    /// axis, or `None` where its part has no direction.
    pub direction: Option<f32>,
    /// The stretch of the line it covers: half its width either way from its centre along the
    /// line, stopping at a Wall's ends and running round a Room past its first point.
    pub stretch: Stretch,
}

/// An edit that renumbers a Wall's segments or a Room's edges.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointEdit {
    /// A point added on a part at a parameter strictly between zero and one.
    Added {
        /// The segment or edge split.
        segment: usize,
        /// Where along it.
        t: f32,
    },
    /// The point of that number removed.
    Removed {
        /// The point.
        index: usize,
    },
}

/// `AnchorPortals`: where each Portal set into the outline `path` stands, its segment being the
/// Wall's segment or the Room's edge, or `None` for one set on a part the outline does not have
/// or at a parameter outside zero to one.
///
/// The centre lies on the exact curve and the direction is the curve's there. The stretch
/// reaches half the Portal's width either way along the line, across the outline's points: on an
/// open line it stops at the ends, and on a closed one it runs round past the first point, a
/// Portal at least as wide as the whole line covering all of it.
#[must_use]
pub fn anchor_portals(path: &Path, portals: &[PortalSetting]) -> Vec<Option<Standing>> {
    let line = flatten(path);
    standings(path, &line, portals)
}

/// Where each Portal set into `path`, flattened into `line`, stands.
fn standings(path: &Path, line: &[LinePoint], portals: &[PortalSetting]) -> Vec<Option<Standing>> {
    let measured = Measured::of(line);
    portals
        .iter()
        .map(|portal| {
            let curve = path.curve(portal.segment)?;
            if !(0.0..=1.0).contains(&portal.t) {
                return None;
            }
            let t = f64::from(portal.t);
            let (centre, direction) = match &curve {
                Curve::Straight(line) => (line.eval(t), line.p1 - line.p0),
                Curve::Bent(quad) => (quad.eval(t), quad.deriv().eval(t).to_vec2()),
            };
            #[expect(
                clippy::cast_possible_truncation,
                reason = "the model keeps angles in single precision"
            )]
            let direction = (length(direction) > NO_DIRECTION)
                .then(|| ops::atan2(direction.y as f32, direction.x as f32));
            let stretch = if path.closed {
                stretch_round(&measured, portal)
            } else {
                stretch_of(&measured, portal)
            };
            Some(Standing {
                centre: vector(centre),
                direction,
                stretch,
            })
        })
        .collect()
}

/// `AnchorPortals` through a point edit: where each Portal set into `path`, as the outline is
/// before the edit, is set once the edit is made, or `None` for a Portal the edit removes.
///
/// Adding a point on part `k` at parameter `s` moves a Portal on part `k` at a parameter `t`
/// below `s` to `t / s` on part `k`, one at or above `s` to `(t - s) / (1 - s)` on part `k + 1`,
/// and a Portal on a later part one part on, so no Portal moves on the Level; a point added on a
/// closed outline's last edge carries the Portals beyond it onto the new last edge.
///
/// Removing a point removes every Portal whose stretch covers it; each other Portal on the two
/// parts it joins goes onto the joined part at the share of their combined length that lay
/// before its centre, and each Portal on a later part one part back. On an open line, removing
/// the first or the last point removes the Portals on the segment it takes away instead, and
/// removing a point of a Wall of two points removes every Portal. On a closed outline the joined
/// edge of a removed first point runs from the last point to the second and is the last edge,
/// and removing a point of a Room of three points removes every Portal. Naming a point the
/// outline does not have leaves every Portal where it is.
#[must_use]
pub fn anchor_portals_through(
    path: &Path,
    edit: PointEdit,
    portals: &[PortalSetting],
) -> Vec<Option<LinePlace>> {
    match edit {
        PointEdit::Added { segment, t } => portals
            .iter()
            .map(|portal| Some(added(portal, segment, t)))
            .collect(),
        PointEdit::Removed { index } if path.closed => removed_round(path, index, portals),
        PointEdit::Removed { index } => removed(path, index, portals),
    }
}

/// Where each Portal goes when the point `index` of a closed outline is removed.
#[expect(
    clippy::cast_possible_truncation,
    reason = "the model keeps parameters in single precision"
)]
fn removed_round(path: &Path, index: usize, portals: &[PortalSetting]) -> Vec<Option<LinePlace>> {
    let points = path.points.len();
    let kept = |portal: &PortalSetting| LinePlace {
        segment: portal.segment,
        t: portal.t,
    };
    if index >= points {
        return portals.iter().map(|portal| Some(kept(portal))).collect();
    }
    if points <= 3 {
        return vec![None; portals.len()];
    }
    let line = flatten(path);
    let measured = Measured::of(&line);
    let total = measured.total();
    // The edge that ends at the point and the edge that starts there, and what they join into.
    let (ending, starting) = ((index + points - 1) % points, index);
    let joined = if index == 0 { points - 2 } else { index - 1 };
    let start_of = |edge: usize| {
        measured.length_at(LinePlace {
            segment: edge,
            t: 0.0,
        })
    };
    let end_of = |edge: usize| {
        measured.length_at(LinePlace {
            segment: edge,
            t: 1.0,
        })
    };
    let ending_length = end_of(ending) - start_of(ending);
    let combined = ending_length + end_of(starting) - start_of(starting);
    let removed_at = start_of(starting);
    portals
        .iter()
        .map(|portal| {
            let stretch = stretch_round(&measured, portal);
            let (from, to) = (
                measured.length_at(stretch.start),
                measured.length_at(stretch.end),
            );
            if covers_round(from, to, removed_at, total) {
                return None;
            }
            let centre = measured.length_at(kept(portal));
            let before = if portal.segment == ending {
                centre - start_of(ending)
            } else if portal.segment == starting {
                ending_length + centre - start_of(starting)
            } else {
                return Some(LinePlace {
                    segment: if portal.segment > index {
                        portal.segment - 1
                    } else {
                        portal.segment
                    },
                    t: portal.t,
                });
            };
            let share = if combined > 0.0 {
                (before / combined).clamp(0.0, 1.0)
            } else {
                0.0
            };
            Some(LinePlace {
                segment: joined,
                t: share as f32,
            })
        })
        .collect()
}

/// Whether the range of arc length from `from` to `to` round a closed line `total` long covers
/// the length `at`, its ends included; a range that starts after it ends runs past the first
/// point, where the lengths zero and `total` are the same place.
fn covers_round(from: f64, to: f64, at: f64, total: f64) -> bool {
    let within = |at: f64| {
        if from <= to {
            from <= at && at <= to
        } else {
            at >= from || at <= to
        }
    };
    within(at) || (at <= 0.0 && within(total)) || (at >= total && within(0.0))
}

/// Where a Portal goes when a point is added on `split` at `s`.
fn added(portal: &PortalSetting, split: usize, s: f32) -> LinePlace {
    let place = LinePlace {
        segment: portal.segment,
        t: portal.t,
    };
    if portal.segment > split {
        return LinePlace {
            segment: portal.segment + 1,
            ..place
        };
    }
    if portal.segment < split || !(s > 0.0 && s < 1.0) {
        return place;
    }
    if portal.t < s {
        LinePlace {
            segment: split,
            t: portal.t / s,
        }
    } else {
        LinePlace {
            segment: split + 1,
            t: (portal.t - s) / (1.0 - s),
        }
    }
}

/// Where each Portal goes when the point `index` of an open line is removed.
#[expect(
    clippy::cast_possible_truncation,
    reason = "the model keeps parameters in single precision"
)]
fn removed(path: &Path, index: usize, portals: &[PortalSetting]) -> Vec<Option<LinePlace>> {
    let points = path.points.len();
    let back = |portal: &PortalSetting| LinePlace {
        segment: portal.segment - 1,
        t: portal.t,
    };
    let kept = |portal: &PortalSetting| LinePlace {
        segment: portal.segment,
        t: portal.t,
    };
    if index >= points {
        return portals.iter().map(|portal| Some(kept(portal))).collect();
    }
    if points <= 2 {
        return vec![None; portals.len()];
    }
    if index == 0 {
        return portals
            .iter()
            .map(|portal| (portal.segment > 0).then(|| back(portal)))
            .collect();
    }
    let last = points - 2;
    if index == points - 1 {
        return portals
            .iter()
            .map(|portal| (portal.segment < last).then(|| kept(portal)))
            .collect();
    }
    let line = flatten(path);
    let measured = Measured::of(&line);
    let at = |segment: usize| measured.length_at(LinePlace { segment, t: 0.0 });
    let (before, removed_at, after) = (
        at(index - 1),
        at(index),
        measured.length_at(LinePlace {
            segment: index,
            t: 1.0,
        }),
    );
    portals
        .iter()
        .map(|portal| {
            let stretch = stretch_of(&measured, portal);
            let (from, to) = (
                measured.length_at(stretch.start),
                measured.length_at(stretch.end),
            );
            if from <= removed_at && removed_at <= to {
                return None;
            }
            Some(if portal.segment > index {
                back(portal)
            } else if portal.segment + 1 == index || portal.segment == index {
                let centre = measured.length_at(kept(portal));
                let joined = after - before;
                let share = if joined > 0.0 {
                    ((centre - before) / joined).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                LinePlace {
                    segment: index - 1,
                    t: share as f32,
                }
            } else {
                kept(portal)
            })
        })
        .collect()
}

/// The stretch a Portal covers: half its width either way from its centre along the measured
/// line, stopping at the line's ends.
fn stretch_of(measured: &Measured, portal: &PortalSetting) -> Stretch {
    let centre = measured.length_at(LinePlace {
        segment: portal.segment,
        t: portal.t,
    });
    let half = f64::from(portal.width) / 2.0;
    Stretch {
        start: measured.place_at(centre - half),
        end: measured.place_at(centre + half),
    }
}

/// The stretch a Portal set into a closed line covers: half its width either way from its
/// centre along the measured line, across the first point; a Portal at least as wide as the
/// whole line covers all of it.
fn stretch_round(measured: &Measured, portal: &PortalSetting) -> Stretch {
    let total = measured.total();
    let centre = measured.length_at(LinePlace {
        segment: portal.segment,
        t: portal.t,
    });
    let half = f64::from(portal.width) / 2.0;
    if 2.0 * half >= total {
        return Stretch {
            start: measured.place_at(0.0),
            end: measured.place_at(total),
        };
    }
    Stretch {
        start: measured.place_at((centre - half).rem_euclid(total)),
        end: measured.place_at((centre + half).rem_euclid(total)),
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::missing_panics_doc,
        reason = "a test stops at the first thing that is not as expected"
    )]

    use super::*;
    use crate::split_wall;

    /// A Wall's open line through `points`, curved where a control is given.
    fn wall(points: &[Vec2], controls: &[Option<Vec2>]) -> Path {
        Path {
            points: points.to_vec(),
            controls: controls.to_vec(),
            closed: false,
        }
    }

    /// The point of the quadratic curve from `start` through `control` to `end` at `t`.
    fn quadratic(start: Vec2, control: Vec2, end: Vec2, t: f32) -> Vec2 {
        let u = 1.0 - t;
        start * (u * u) + control * (2.0 * u * t) + end * (t * t)
    }

    /// The point of a Wall at a place of its line, on its exact curve.
    fn on_wall(wall: &Path, place: LinePlace) -> Vec2 {
        let (start, end) = (wall.points[place.segment], wall.points[place.segment + 1]);
        match wall.controls[place.segment] {
            None => start.lerp(end, place.t),
            Some(control) => quadratic(start, control, end, place.t),
        }
    }

    /// One Portal's standing, which it must have.
    fn standing(wall: &Path, segment: usize, t: f32, width: f32) -> Standing {
        anchor_portals(wall, &[PortalSetting { segment, t, width }])[0]
            .expect("the Portal is set into the Wall")
    }

    /// The centre lies on the quadratic curve at the Portal's parameter, turned to the curve's
    /// direction there, and a direction of no length is none.
    #[test]
    fn the_centre_lies_on_the_curve() {
        let (start, control, end) = (Vec2::ZERO, Vec2::new(2.0, 4.0), Vec2::new(4.0, 0.0));
        let bent = wall(&[start, end], &[Some(control)]);

        let middle = standing(&bent, 0, 0.5, 1.0);
        assert!(middle.centre.distance(Vec2::new(2.0, 2.0)) < 1e-6);
        assert!(middle.direction.expect("a direction").abs() < 1e-6);

        let quarter = standing(&bent, 0, 0.25, 1.0);
        assert!(
            quarter
                .centre
                .distance(quadratic(start, control, end, 0.25))
                < 1e-6
        );
        let expected = ops::atan2(1.0, 1.0);
        assert!((quarter.direction.expect("a direction") - expected).abs() < 1e-6);

        let pinched = wall(&[start, end], &[Some(end)]);
        assert_eq!(standing(&pinched, 0, 1.0, 1.0).direction, None);
        assert_eq!(
            anchor_portals(
                &bent,
                &[PortalSetting {
                    segment: 1,
                    t: 0.5,
                    width: 1.0
                }]
            ),
            vec![None]
        );
    }

    /// A stretch reaches half the width either way along the line, across a point of the Wall,
    /// and stops at the Wall's ends.
    #[test]
    fn a_stretch_crosses_points_and_stops_at_ends() {
        let corner = wall(
            &[Vec2::ZERO, Vec2::new(4.0, 0.0), Vec2::new(4.0, 4.0)],
            &[None, None],
        );

        let across = standing(&corner, 0, 0.875, 2.0).stretch;
        assert_eq!(across.start.segment, 0);
        assert!((across.start.t - 0.625).abs() < 1e-6);
        assert_eq!(across.end.segment, 1);
        assert!((across.end.t - 0.125).abs() < 1e-6);

        let at_the_start = standing(&corner, 0, 0.125, 2.0).stretch;
        assert_eq!(at_the_start.start, LinePlace { segment: 0, t: 0.0 });
        assert!((at_the_start.end.t - 0.375).abs() < 1e-6);

        let at_the_end = standing(&corner, 1, 0.875, 2.0).stretch;
        assert_eq!(at_the_end.end, LinePlace { segment: 1, t: 1.0 });
        assert!((at_the_end.start.t - 0.625).abs() < 1e-6);
    }

    /// A point added on a Portal's segment, before or after it, or on an earlier segment, leaves
    /// the Portal's centre where it was, on a straight and on a curved segment.
    #[test]
    fn an_added_point_keeps_the_centre() {
        let before = wall(
            &[Vec2::ZERO, Vec2::new(4.0, 0.0), Vec2::new(8.0, 0.0)],
            &[None, Some(Vec2::new(6.0, 3.0))],
        );
        let portals = [
            PortalSetting {
                segment: 0,
                t: 0.3,
                width: 1.0,
            },
            PortalSetting {
                segment: 1,
                t: 0.2,
                width: 1.0,
            },
            PortalSetting {
                segment: 1,
                t: 0.7,
                width: 1.0,
            },
        ];
        let centres: Vec<Vec2> = anchor_portals(&before, &portals)
            .into_iter()
            .map(|standing| standing.expect("set").centre)
            .collect();
        for (segment, s) in [(1, 0.4), (0, 0.5), (0, 0.2)] {
            let after = split_wall(&before, segment, s).expect("the split");
            let places =
                anchor_portals_through(&before, PointEdit::Added { segment, t: s }, &portals);
            for (place, centre) in places.into_iter().zip(&centres) {
                let place = place.expect("an added point removes no Portal");
                let moved = on_wall(&after, place);
                assert!(
                    moved.distance(*centre) < 1e-5,
                    "{moved} against {centre} after a point on {segment} at {s}"
                );
            }
        }
    }

    /// A Room's closed outline through `points`, every edge straight.
    fn room(points: &[Vec2]) -> Path {
        Path {
            points: points.to_vec(),
            controls: vec![None; points.len()],
            closed: true,
        }
    }

    /// A square Room four cells a side, every edge straight, from the origin counter-clockwise.
    fn square() -> Path {
        room(&[
            Vec2::ZERO,
            Vec2::new(4.0, 0.0),
            Vec2::new(4.0, 4.0),
            Vec2::new(0.0, 4.0),
        ])
    }

    /// A stretch of a Room's Walls runs on past the first point, starting on the last edge and
    /// ending on the first, and a Portal wider than the whole outline covers all of it.
    #[test]
    fn a_stretch_wraps_past_the_first_point() {
        let room = square();
        let across = anchor_portals(
            &room,
            &[PortalSetting {
                segment: 0,
                t: 0.125,
                width: 2.0,
            }],
        )[0]
        .expect("the Portal is set into the Room");
        assert_eq!(across.stretch.start.segment, 3);
        assert!((across.stretch.start.t - 0.875).abs() < 1e-6);
        assert_eq!(across.stretch.end.segment, 0);
        assert!((across.stretch.end.t - 0.375).abs() < 1e-6);
        assert!(across.stretch.covers(LinePlace { segment: 0, t: 0.0 }));
        assert!(across.stretch.covers(LinePlace { segment: 3, t: 1.0 }));
        assert!(!across.stretch.covers(LinePlace { segment: 1, t: 0.5 }));

        let wide = anchor_portals(
            &room,
            &[PortalSetting {
                segment: 2,
                t: 0.5,
                width: 20.0,
            }],
        )[0]
        .expect("the Portal is set into the Room");
        for segment in 0..4 {
            assert!(
                wide.stretch.covers(LinePlace { segment, t: 0.5 }),
                "{segment}"
            );
        }
    }

    /// Removing a Room's first point joins its last edge and its first into one, the new last
    /// edge from the last point to the second: a Portal on either goes onto it at its share of
    /// their combined length, a Portal on a later edge one edge back, and a Portal covering the
    /// point goes.
    #[test]
    fn removing_the_first_point_remaps_round_the_outline() {
        let square = square();
        let portals = [
            PortalSetting {
                segment: 3,
                t: 0.5,
                width: 1.0,
            },
            PortalSetting {
                segment: 0,
                t: 0.5,
                width: 1.0,
            },
            PortalSetting {
                segment: 1,
                t: 0.25,
                width: 1.0,
            },
            PortalSetting {
                segment: 0,
                t: 0.0625,
                width: 1.0,
            },
        ];
        let places = anchor_portals_through(&square, PointEdit::Removed { index: 0 }, &portals);

        let joined = places[0].expect("the Portal on the last edge stays");
        assert_eq!(joined.segment, 2);
        assert!((joined.t - 0.25).abs() < 1e-6);
        let joined = places[1].expect("the Portal on the first edge stays");
        assert_eq!(joined.segment, 2);
        assert!((joined.t - 0.75).abs() < 1e-6);
        assert_eq!(
            places[2],
            Some(LinePlace {
                segment: 0,
                t: 0.25
            })
        );
        assert_eq!(places[3], None, "the Portal over the first point goes");

        let three = room(&[Vec2::ZERO, Vec2::X, Vec2::Y]);
        assert_eq!(
            anchor_portals_through(&three, PointEdit::Removed { index: 1 }, &portals[..1]),
            vec![None]
        );
    }
}
