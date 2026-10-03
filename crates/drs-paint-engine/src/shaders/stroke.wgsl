// Strokes rasterized into one coverage tile: one quad per segment of a stroke's path, covering
// the segment's capsule, whose fragments each give that segment's coverage at their pixel; the
// target's blend keeps the largest for a stroke that paints and the smallest of one minus it for
// an erase, so a stroke's coverage is the largest any of its segments gives, as on the CPU.
//
// Plain WGSL with no imports. `Tile` mirrors the paint Engine's `TileData` and `Segment` its
// `SegmentData`, field by field. A pixel's value is taken at its centre, found from its
// whole-pixel index in the Level's pixel plane divided by the pixels per cell, row 0 at the
// tile's top, with the same arithmetic as the CPU rasterizer.

struct Tile {
    // The whole-pixel index of the tile's lower-left pixel along each axis.
    origin: vec2<i32>,
    // How many pixels one Grid cell spans.
    band: f32,
    padding: f32,
};

struct Segment {
    // Where it starts, in Grid cells.
    start: vec2<f32>,
    // Where it ends; the same point for a one-point path.
    end: vec2<f32>,
    // The Brush's radius, in Grid cells.
    radius: f32,
    // How much of the radius shows the full strength.
    hardness: f32,
    // How much of the Material the stroke shows where it shows the most.
    strength: f32,
    padding: f32,
};

@group(0) @binding(0) var<uniform> tile: Tile;
@group(0) @binding(1) var<storage, read> segments: array<Segment>;

// How many pixels a side a tile has.
const SIDE: f32 = 512.0;

struct Corner {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) segment: u32,
};

// One corner of the quad of the segment `instance`: two triangles around its capsule, a pixel
// wider than the radius on every side, in the tile's pixels.
@vertex
fn capsule(@builtin(vertex_index) vertex: u32, @builtin(instance_index) instance: u32) -> Corner {
    let segment = segments[instance];
    let origin = vec2<f32>(tile.origin);
    let start = segment.start * tile.band - origin;
    let end = segment.end * tile.band - origin;
    let along = end - start;
    let span = length(along);
    var direction = vec2<f32>(1.0, 0.0);
    if span > 0.0 {
        direction = along / span;
    }
    let across = vec2<f32>(-direction.y, direction.x);
    let reach = segment.radius * tile.band + 1.0;
    let first = start - direction * reach;
    let last = end + direction * reach;
    var corners = array<vec2<f32>, 6>(
        first - across * reach,
        last - across * reach,
        last + across * reach,
        first - across * reach,
        last + across * reach,
        first + across * reach,
    );
    var corner: Corner;
    // The tile's bottom is the target's last row, so its top is row 0.
    corner.position = vec4<f32>(corners[vertex] / (SIDE / 2.0) - 1.0, 0.0, 1.0);
    corner.segment = instance;
    return corner;
}

// The coverage the segment gives at the centre of the pixel a fragment covers.
fn coverage(corner: Corner) -> f32 {
    let segment = segments[corner.segment];
    let column = i32(floor(corner.position.x));
    let row = i32(floor(corner.position.y));
    let index = vec2<i32>(tile.origin.x + column, tile.origin.y + i32(SIDE) - 1 - row);
    let centre = (vec2<f32>(index) + 0.5) / tile.band;
    let along = segment.end - segment.start;
    let to = centre - segment.start;
    let span = along.x * along.x + along.y * along.y;
    var t = 0.0;
    if span > 0.0 {
        t = clamp((to.x * along.x + to.y * along.y) / span, 0.0, 1.0);
    }
    let off = vec2<f32>(to.x - along.x * t, to.y - along.y * t);
    let distance = sqrt(off.x * off.x + off.y * off.y);
    let inner = segment.hardness * segment.radius;
    if distance <= inner {
        return segment.strength;
    }
    if distance >= segment.radius {
        return 0.0;
    }
    let x = (distance - inner) / (segment.radius - inner);
    return segment.strength * (1.0 - 3.0 * x * x + 2.0 * x * x * x);
}

// A stroke that paints: the target keeps the larger of what it holds and this.
@fragment
fn paint(corner: Corner) -> @location(0) vec4<f32> {
    return vec4<f32>(coverage(corner));
}

// An erase: the target keeps the smaller of what it holds and one minus the coverage.
@fragment
fn erase(corner: Corner) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0 - coverage(corner));
}
