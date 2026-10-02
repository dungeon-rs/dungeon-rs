// A Terrain's Material over one square of its coverage: its image repeated edge to edge across
// the Level at its natural size, masked by the coverage.
//
// Plain WGSL with no imports: the fragment stage reads only the texture coordinates of the 2D
// mesh vertex shader's output, and the Material's bind group is group 2. `VertexOutput` mirrors
// the part read of the `VertexOutput` of Bevy's 2D mesh vertex shader, in `bevy_sprite_render`'s
// `mesh2d/vertex_output.wesl`, and `Params` mirrors the render Engine's `Params` field by field.

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(2) uv: vec2<f32>,
};

struct Params {
    // The image's natural size in Grid cells: one repetition.
    image_cells: vec2<f32>,
    // The lower-left corner of the square drawn, in Grid cells.
    origin: vec2<f32>,
    // The width and height of the square drawn, in Grid cells.
    extent: vec2<f32>,
    // 1 to draw the flat colour in place of the image, 0 to draw the image.
    flat: f32,
    // The flat colour, linear, with its own opacity.
    colour: vec4<f32>,
};

@group(2) @binding(0) var<uniform> params: Params;
@group(2) @binding(1) var image: texture_2d<f32>;
@group(2) @binding(2) var image_sampler: sampler;
@group(2) @binding(3) var coverage: texture_2d<f32>;
@group(2) @binding(4) var coverage_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // The mesh's texture coordinates run from the top-left corner, as the coverage's rows do.
    let covered = textureSample(coverage, coverage_sampler, in.uv).r;
    let cell = params.origin + vec2<f32>(in.uv.x, 1.0 - in.uv.y) * params.extent;
    // One repetition has its lower-left corner at the Level's origin; the image's rows run
    // from its top.
    let repeated = fract(cell / params.image_cells);
    let texel = textureSample(image, image_sampler, vec2<f32>(repeated.x, 1.0 - repeated.y));
    let shown = select(texel, params.colour, params.flat > 0.5);
    return vec4<f32>(shown.rgb, shown.a * covered);
}
