//! `Rasterize`: the coverage of strokes over a region of the Level, one byte a pixel, on the CPU.
//!
//! The rasterizer is the golden reference: every pixel's value depends only on the strokes, the
//! pixel's position in the Level's pixel plane, and the pixels per cell, never on the region it
//! is computed in, and only square roots and arithmetic go into it, so it is the same on every
//! machine.

use bevy_math::{UVec2, Vec2};
use drs_model::Stroke;

/// The coverage of `strokes` over the region whose lower-left corner is `corner`, in Grid cells,
/// and which is `size` pixels wide and high at `pixels_per_cell`: one byte a pixel, rows from the
/// region's top, 255 where the coverage is full.
///
/// A pixel's coverage is taken at its centre, which is found from its whole-pixel index in the
/// Level's pixel plane (the corner in pixels, rounded to the nearest whole pixel, plus its index
/// in the region) divided by the pixels per cell once per axis, so a pixel has the same value in
/// every region that holds it. Each stroke's coverage at the centre is its strength where the
/// distance to its path is within the hardness of its radius, falls off smoothly to nothing at
/// the radius, and is the same however many of its segments pass near. The region starts empty
/// and the strokes are composited in order: a stroke that paints raises a pixel to its coverage
/// where that is higher, and an erase lowers it to one minus its coverage where that is lower. A
/// value is the coverage, or one minus an erase's, times 255, rounded half up.
#[must_use]
pub fn rasterize(strokes: &[Stroke], corner: Vec2, size: UVec2, pixels_per_cell: u32) -> Vec<u8> {
    let mut pixels = vec![0; size.x as usize * size.y as usize];
    let region = Region::at(corner, size, pixels_per_cell);
    for stroke in strokes {
        composite(&mut pixels, stroke, &region);
    }
    pixels
}

/// A region of the Level's pixel plane at a resolution.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Region {
    /// The whole-pixel index of the region's lower-left pixel along each axis.
    origin: [i64; 2],
    /// The width and height in pixels.
    size: [i64; 2],
    /// How many pixels one cell spans.
    pixels_per_cell: f32,
}

impl Region {
    /// The region whose lower-left corner is `corner` in cells, rounded to the nearest whole
    /// pixel, `size` pixels across at `pixels_per_cell`.
    pub(crate) fn at(corner: Vec2, size: UVec2, pixels_per_cell: u32) -> Self {
        #[expect(
            clippy::cast_precision_loss,
            reason = "a resolution is at most a few thousand pixels per cell"
        )]
        let pixels_per_cell = pixels_per_cell as f32;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a corner in pixels is far inside the range of i64"
        )]
        let whole = |cells: f32| (cells * pixels_per_cell).round() as i64;
        Self {
            origin: [whole(corner.x), whole(corner.y)],
            size: [i64::from(size.x), i64::from(size.y)],
            pixels_per_cell,
        }
    }

    /// The centre of the pixel of whole-pixel index `index`, in cells along one axis.
    fn centre(&self, index: i64) -> f32 {
        #[expect(
            clippy::cast_precision_loss,
            reason = "pixel indices are far below where f32 loses whole numbers"
        )]
        let pixels = index as f32 + 0.5;
        pixels / self.pixels_per_cell
    }

    /// The whole-pixel indices along one axis whose centres may lie from `low` to `high` cells,
    /// a pixel wider either way, within the region.
    fn span(&self, axis: usize, low: f32, high: f32) -> std::ops::Range<i64> {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a pixel index is far inside the range of i64"
        )]
        let index = |cells: f32| (cells * self.pixels_per_cell - 0.5) as i64;
        let start = (index(low) - 1).max(self.origin[axis]);
        let end = (index(high) + 2).min(self.origin[axis] + self.size[axis]);
        start..end.max(start)
    }
}

/// A straight segment of a stroke's path and the box it reaches, grown by the radius.
#[derive(Debug, Clone, Copy)]
struct Segment {
    /// Where it starts.
    from: Vec2,
    /// Where it ends; the same point for a one-point path.
    to: Vec2,
    /// The lower-left corner of its box grown by the radius.
    low: Vec2,
    /// The upper-right corner of that box.
    high: Vec2,
}

impl Segment {
    /// The segment from `from` to `to` for a Brush of `radius`.
    fn new(from: Vec2, to: Vec2, radius: f32) -> Self {
        Self {
            from,
            to,
            low: Vec2::new(from.x.min(to.x) - radius, from.y.min(to.y) - radius),
            high: Vec2::new(from.x.max(to.x) + radius, from.y.max(to.y) + radius),
        }
    }

    /// The square of the distance from `(x, y)` to the nearest point of the segment.
    fn distance_squared(&self, x: f32, y: f32) -> f32 {
        let (along_x, along_y) = (self.to.x - self.from.x, self.to.y - self.from.y);
        let (to_x, to_y) = (x - self.from.x, y - self.from.y);
        let length = along_x * along_x + along_y * along_y;
        let t = if length > 0.0 {
            ((to_x * along_x + to_y * along_y) / length).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let (off_x, off_y) = (to_x - along_x * t, to_y - along_y * t);
        off_x * off_x + off_y * off_y
    }
}

/// The segments of a stroke's path: one from each point to the next, or a single one of no
/// length for a one-point path.
fn segments(stroke: &Stroke) -> Vec<Segment> {
    let radius = stroke.brush.radius();
    match stroke.points.as_slice() {
        [] => Vec::new(),
        [point] => vec![Segment::new(*point, *point, radius)],
        points => points
            .windows(2)
            .map(|pair| Segment::new(pair[0], pair[1], radius))
            .collect(),
    }
}

/// A stroke's coverage at distance `distance` from its path, before it is turned into a byte.
pub(crate) fn stamp(stroke: &Stroke, distance: f32) -> f32 {
    let radius = stroke.brush.radius();
    let inner = stroke.brush.hardness * radius;
    if distance <= inner {
        stroke.brush.strength
    } else if distance >= radius {
        0.0
    } else {
        let t = (distance - inner) / (radius - inner);
        stroke.brush.strength * (1.0 - 3.0 * t * t + 2.0 * t * t * t)
    }
}

/// A coverage as a byte: times 255, rounded half up.
fn byte(coverage: f32) -> u8 {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the value is clamped to the range of a byte first"
    )]
    let value = (coverage * 255.0 + 0.5).floor().clamp(0.0, 255.0) as u8;
    value
}

/// Composites `stroke` onto `pixels`, the region's coverage: a stroke that paints raises each
/// pixel to its coverage where that is larger, and an erase lowers each to one minus its coverage
/// where that is smaller, its coverage the largest any of its segments gives before it is
/// applied. Only the pixels inside the stroke's box are visited, an erase's too, and for each
/// only the segments whose own box holds its centre.
pub(crate) fn composite(pixels: &mut [u8], stroke: &Stroke, region: &Region) {
    let segments = segments(stroke);
    let Some((low, high)) = segments
        .iter()
        .map(|segment| (segment.low, segment.high))
        .reduce(|(low, high), (segment_low, segment_high)| {
            (low.min(segment_low), high.max(segment_high))
        })
    else {
        return;
    };
    let width = region.size[0];
    let mut near: Vec<&Segment> = Vec::with_capacity(segments.len());
    for row in region.span(1, low.y, high.y) {
        let y = region.centre(row);
        near.clear();
        near.extend(
            segments
                .iter()
                .filter(|segment| segment.low.y <= y && y <= segment.high.y),
        );
        if near.is_empty() {
            continue;
        }
        let from_top = region.size[1] - 1 - (row - region.origin[1]);
        let Ok(start) = usize::try_from(from_top * width) else {
            continue;
        };
        for column in region.span(0, low.x, high.x) {
            let x = region.centre(column);
            let nearest = near
                .iter()
                .filter(|segment| segment.low.x <= x && x <= segment.high.x)
                .map(|segment| segment.distance_squared(x, y))
                .reduce(f32::min);
            let Some(nearest) = nearest else {
                continue;
            };
            let coverage = stamp(stroke, nearest.sqrt());
            let Ok(offset) = usize::try_from(column - region.origin[0]) else {
                continue;
            };
            let Some(pixel) = pixels.get_mut(start + offset) else {
                continue;
            };
            *pixel = if stroke.erase {
                (*pixel).min(byte(1.0 - coverage))
            } else {
                (*pixel).max(byte(coverage))
            };
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::missing_panics_doc,
        clippy::float_cmp,
        reason = "a test stops at the first thing that is not as expected, and the values \
                  asserted on are exact"
    )]

    use super::*;
    use drs_model::BrushSettings;

    /// A stroke through `points` with a Brush of `size`, `hardness`, and `strength`.
    fn stroke(points: &[Vec2], size: f32, hardness: f32, strength: f32) -> Stroke {
        Stroke {
            points: points.to_vec(),
            brush: BrushSettings {
                size,
                hardness,
                strength,
            },
            erase: false,
        }
    }

    /// The same stroke, erasing.
    fn erasing(stroke: Stroke) -> Stroke {
        Stroke {
            erase: true,
            ..stroke
        }
    }

    /// The byte at a column and a row counted from the bottom of a region `size` pixels across.
    fn at(pixels: &[u8], size: UVec2, column: u32, row: u32) -> u8 {
        pixels[((size.y - 1 - row) * size.x + column) as usize]
    }

    /// The stamp is the strength within the hardness of the radius, nothing beyond the radius,
    /// and the smooth falloff between, as its formula says.
    #[test]
    fn the_stamp_matches_its_formula() {
        let dab = stroke(&[Vec2::ZERO], 4.0, 0.25, 0.8);
        assert_eq!(stamp(&dab, 0.0), 0.8);
        assert_eq!(stamp(&dab, 0.5), 0.8);
        assert_eq!(stamp(&dab, 2.0), 0.0);
        assert_eq!(stamp(&dab, 3.0), 0.0);
        let halfway = stamp(&dab, 1.25);
        assert!((halfway - 0.4).abs() < 1e-6, "{halfway}");
        let t = 0.25_f32;
        let quarter = stamp(&dab, 0.5 + 1.5 * t);
        let expected = 0.8 * (1.0 - 3.0 * t * t + 2.0 * t * t * t);
        assert!(
            (quarter - expected).abs() < 1e-6,
            "{quarter} against {expected}"
        );

        let hard = stroke(&[Vec2::ZERO], 2.0, 1.0, 1.0);
        assert_eq!(stamp(&hard, 1.0), 1.0);
        assert_eq!(stamp(&hard, 1.000_001), 0.0);

        // A dab at a pixel's centre: full there, nothing a radius away.
        let size = UVec2::splat(8);
        let pixels = rasterize(
            &[stroke(&[Vec2::splat(0.125 + 1.0 / 16.0)], 0.25, 0.5, 1.0)],
            Vec2::ZERO,
            size,
            8,
        );
        assert_eq!(at(&pixels, size, 1, 1), 255);
        assert_eq!(at(&pixels, size, 3, 1), 0);
    }

    /// At a sharp joint the coverage is the largest any segment gives, not their sum.
    #[test]
    fn a_joint_takes_the_maximum() {
        let size = UVec2::new(64, 64);
        let joint = Vec2::splat(4.0 + 1.0 / 16.0);
        let bent = stroke(
            &[Vec2::new(0.5, 0.5), joint, Vec2::new(7.5, 0.5)],
            3.0,
            0.2,
            0.6,
        );
        let pixels = rasterize(std::slice::from_ref(&bent), Vec2::ZERO, size, 8);
        let first = rasterize(
            &[stroke(&bent.points[..2], 3.0, 0.2, 0.6)],
            Vec2::ZERO,
            size,
            8,
        );
        let second = rasterize(
            &[stroke(&bent.points[1..], 3.0, 0.2, 0.6)],
            Vec2::ZERO,
            size,
            8,
        );
        for (index, pixel) in pixels.iter().enumerate() {
            assert_eq!(*pixel, first[index].max(second[index]), "pixel {index}");
        }
        assert_eq!(at(&pixels, size, 32, 32), byte(0.6));
    }

    /// A pixel has the same value whichever region it is computed in, at any corner and size.
    #[test]
    fn a_pixel_is_the_same_in_any_region() {
        let strokes = [
            stroke(
                &[
                    Vec2::new(-3.3, 1.7),
                    Vec2::new(2.9, -0.4),
                    Vec2::new(4.1, 3.3),
                ],
                1.7,
                0.4,
                0.9,
            ),
            stroke(&[Vec2::new(1.0, 1.0)], 2.5, 0.0, 0.5),
        ];
        for pixels_per_cell in [7, 32, 100] {
            let whole = rasterize(
                &strokes,
                Vec2::splat(-5.0),
                UVec2::splat(10 * pixels_per_cell),
                pixels_per_cell,
            );
            let side = 10 * pixels_per_cell;
            #[expect(clippy::cast_precision_loss, reason = "small whole numbers")]
            let cells = |pixels: u32| pixels as f32 / pixels_per_cell as f32;
            for (left, bottom, width, height) in [
                (0, 0, side, side),
                (13, 21, 50, 33),
                (side - 40, side - 9, 40, 9),
            ] {
                let part = rasterize(
                    &strokes,
                    Vec2::new(-5.0 + cells(left), -5.0 + cells(bottom)),
                    UVec2::new(width, height),
                    pixels_per_cell,
                );
                for row in 0..height {
                    for column in 0..width {
                        assert_eq!(
                            at(&part, UVec2::new(width, height), column, row),
                            at(&whole, UVec2::splat(side), left + column, bottom + row),
                            "pixel ({column}, {row}) of the region at ({left}, {bottom}) at {pixels_per_cell} px per cell"
                        );
                    }
                }
            }
        }
    }

    /// An erase lowers each pixel to one minus its coverage, times 255 rounded half up, where that
    /// is lower than what the strokes before it left, and leaves the rest: the smaller of the two
    /// bytes everywhere, the very byte a half-strength erase finds over half-strength paint, and
    /// nothing added where nothing was painted.
    #[test]
    fn an_erase_takes_the_minimum() {
        let size = UVec2::splat(64);
        let path = [
            Vec2::new(1.0, 4.0 + 1.0 / 16.0),
            Vec2::new(7.0, 4.0 + 1.0 / 16.0),
        ];
        let across = [
            Vec2::new(4.0 + 1.0 / 16.0, 1.0),
            Vec2::new(4.0 + 1.0 / 16.0, 7.0),
        ];
        for (paint, erase) in [
            (stroke(&path, 2.0, 0.5, 1.0), stroke(&across, 3.0, 0.3, 1.0)),
            (stroke(&path, 2.0, 1.0, 0.5), stroke(&across, 3.0, 1.0, 0.5)),
            (stroke(&path, 4.0, 0.0, 0.8), stroke(&across, 2.5, 0.2, 0.6)),
        ] {
            let painted = rasterize(std::slice::from_ref(&paint), Vec2::ZERO, size, 8);
            let erased = rasterize(
                &[paint.clone(), erasing(erase.clone())],
                Vec2::ZERO,
                size,
                8,
            );
            for row in 0..size.y {
                for column in 0..size.x {
                    #[expect(clippy::cast_precision_loss, reason = "small whole numbers")]
                    let centre = Vec2::new(column as f32 + 0.5, row as f32 + 0.5) / 8.0;
                    let distance = if (across[0].y..=across[1].y).contains(&centre.y) {
                        (centre.x - across[0].x).abs()
                    } else {
                        across[0].distance(centre).min(across[1].distance(centre))
                    };
                    let left = byte(1.0 - stamp(&erase, distance));
                    let found = at(&painted, size, column, row);
                    assert_eq!(
                        at(&erased, size, column, row),
                        found.min(left),
                        "pixel ({column}, {row}) under {erase:?}"
                    );
                }
            }
        }
        let half = rasterize(
            &[
                stroke(&path, 2.0, 1.0, 0.5),
                erasing(stroke(&across, 3.0, 1.0, 0.5)),
            ],
            Vec2::ZERO,
            size,
            8,
        );
        assert_eq!(at(&half, size, 32, 32), byte(0.5), "half over half");
        let alone = rasterize(
            &[erasing(stroke(&across, 3.0, 1.0, 1.0))],
            Vec2::ZERO,
            size,
            8,
        );
        assert!(
            alone.iter().all(|pixel| *pixel == 0),
            "an erase adds nothing"
        );
    }
}
