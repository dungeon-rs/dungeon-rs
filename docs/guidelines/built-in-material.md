# Built-in Material with a fixed-handle Shader

**Use when**: an Engine draws with a Shader of its own that ships with the editor, under a fixed handle: RenderEngine's Material (the masked tiled image a Terrain is drawn with) or a render-world pipeline (PaintEngine's stroke Shader, which follows the Shader rules here and leaves the Material ones aside). **Not when**: the Shader comes from an Asset or a Plugin (an author Shader, which the Shader contract and naga validation at load time govern), or a Material Bevy already has does the job (`ColorMaterial` for a Wall's flat colour).
**Exemplar**: `crates/drs-render-engine/src/terrain.rs`

## Rules

- The Shader is a Bundled File: plain WGSL with no Bevy imports in `src/shaders/<name>.wgsl`, compiled in with `include_str!`. Its fragment input names the Bevy struct it mirrors and declares only the fields it reads at their locations; its `Params` mirrors the Rust `Params` field by field in the same order. The `wgsl-shaders` check of `just workspace` parses and validates every such file with naga.
- The Shader has a fixed handle, a `const` made with `uuid_handle!` from a fresh UUID, and the Material's `fragment_shader` returns `ShaderRef::Handle` of it, so nothing loads it through the asset server.
- The plugin inserts the Shader into `Assets<Shader>` in `Plugin::finish`, not `build`, through an `add_shader` function that takes `Option<&mut Assets<Shader>>` and logs when there are none, since `build` runs before the render plugins have made the shader assets when the plugin is added first. An Engine that can do without the Shader has it return whether it was added and falls back in `finish` when it was not (PaintEngine then hands no work to the render world and rasterizes on the CPU).
- The Material derives `Asset`, `TypePath`, and `AsBindGroup`, keeps its bind group at group 2 with the uniform at binding 0, and is registered with `Material2dPlugin::<M>` in `build`; it blends (`AlphaMode2d::Blend`) when it must sort with the sprites and meshes by depth.
- When an image a Material binds is replaced in `Assets<Image>`, the Material is marked changed too (`get_mut(..)` and `into_inner()`), since its bind group keeps the texture it was prepared with.

## Example

```rust
/// The masked tiled image Shader, a Bundled File compiled into the Engine.
const SHADER_SOURCE: &str = include_str!("shaders/terrain.wgsl");

/// The handle the masked tiled image Shader is added under at startup.
pub(crate) const SHADER: Handle<Shader> = uuid_handle!("5d7c2f0e-8b4a-4c61-9e3f-2a6d1b8c4f17");

/// Adds the masked tiled image Shader to the shader assets, or says that Terrain is not drawn
/// when there are none.
pub(crate) fn add_shader(shaders: Option<&mut Assets<Shader>>) {
    let Some(shaders) = shaders else {
        log::warn!(
            "there are no shader assets to add the Terrain Shader to, so Terrain is not drawn"
        );
        return;
    };
    if shaders
        .insert(
            &SHADER,
            Shader::from_wgsl(SHADER_SOURCE, "drs-render-engine/terrain.wgsl"),
        )
        .is_err()
    {
        log::warn!("the Terrain Shader could not be added, so Terrain is not drawn");
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(crate) struct TerrainMaterial {
    /// Where the square drawn lies, how large one repetition is, and whether the flat colour
    /// stands in for the image.
    #[uniform(0)]
    params: Params,
    /// The image, or `None` while the flat colour stands in.
    #[texture(1)]
    #[sampler(2)]
    image: Option<Handle<Image>>,
    /// The coverage of the square drawn, one byte a texel, rows from the top.
    #[texture(3)]
    #[sampler(4)]
    coverage: Handle<Image>,
}

impl Material2d for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(SHADER)
    }

    /// It blends, so that a Terrain sorts with the sprites and the Walls' meshes by depth.
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// The Material's uniform, as the Shader declares it.
#[derive(ShaderType, Debug, Clone, Copy, PartialEq)]
struct Params {
    /// The image's natural size in Grid cells.
    image_cells: Vec2,
    /// The lower-left corner of the square drawn, in Grid cells.
    origin: Vec2,
    /// The width and height of the square drawn, in Grid cells.
    extent: Vec2,
    /// 1 to draw the flat colour in place of the image, 0 to draw the image.
    flat: f32,
    /// The flat colour, linear.
    colour: Vec4,
}
```

## Pitfalls

- Importing Bevy's shader modules into the WGSL: Bevy 0.20 moved its own shaders to WESL and renamed their paths, so such a Shader breaks on the next upgrade.
- Inserting the Shader in `build` when the shader assets exist: it works or silently does nothing depending on plugin order, and Terrain is then never drawn.
- Replacing a bound image without touching the Material: the viewport goes on showing the old pixels until something else changes the Material.
