//! Terrain: each painted Element drawn as its Material masked by its coverage, one quad per
//! coverage tile with the masked tiled image Material, kept in step with the model through change
//! detection.

use crate::drawn_as;
use crate::projection::VIEWPORT_LAYER;
use crate::stacking::Stacking;
use bevy_asset::{
    Asset, AssetId, AssetIndex, AssetPath, AssetServer, Assets, Handle, LoadState,
    RenderAssetUsages, uuid_handle,
};
use bevy_camera::visibility::RenderLayers;
use bevy_color::{Color, ColorToComponents, LinearRgba};
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::Children;
use bevy_ecs::lifecycle::RemovedComponents;
use bevy_ecs::query::{Changed, Or, With};
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{Commands, Query, Res, ResMut, SystemParam};
use bevy_image::{Image, ImageSampler};
use bevy_math::{Vec2, Vec3, Vec4};
use bevy_mesh::{Indices, Mesh, Mesh2d, PrimitiveTopology};
use bevy_reflect::TypePath;
use bevy_render::render_resource::{
    AsBindGroup, Extent3d, ShaderType, TextureDimension, TextureFormat,
};
use bevy_shader::{Shader, ShaderRef};
use bevy_sprite_render::{AlphaMode2d, Material2d, MeshMaterial2d};
use bevy_transform::components::Transform;
use drs_library_access::asset_path;
use drs_model::{
    AssetReferences, COVERAGE_PIXELS_PER_CELL, COVERAGE_TILE_PIXELS, CoverageTile, DrawnAs,
    Element, ElementKindRegistry, GpuTile, Grid, Layer as ModelLayer, Level, Project, Resolution,
    ResolutionTable, Terrain, TerrainCoverage, TileContent, TileKey, tile_cells,
};
use std::collections::BTreeMap;

/// The masked tiled image Shader, a Bundled File compiled into the Engine.
const SHADER_SOURCE: &str = include_str!("shaders/terrain.wgsl");

/// The handle the masked tiled image Shader is added under at startup.
pub(crate) const SHADER: Handle<Shader> = uuid_handle!("5d7c2f0e-8b4a-4c61-9e3f-2a6d1b8c4f17");

/// The flat colour that stands in for a Terrain's image while it loads, when it is Missing, and
/// when it failed to load, masked by the coverage as the image would be.
const PLACEHOLDER: Color = Color::srgba(0.75, 0.3, 0.35, 0.8);

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

/// The masked tiled image: a Terrain's image repeated edge to edge across the Level at its
/// natural size, upright, one repetition with its lower-left corner at the Level's origin,
/// as opaque at each point as the coverage there times the image's own opacity.
///
/// The Shader wraps the image's coordinates itself and samples it with the image's own sampler,
/// so a Prop and a Terrain share one loaded image.
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

/// How a Terrain looks wherever it is drawn: its image once loaded, or the flat placeholder.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Look {
    /// The image, or `None` while the flat colour stands in.
    image: Option<Handle<Image>>,
    /// One repetition of the image, in Grid cells.
    image_cells: Vec2,
}

impl Look {
    /// The flat placeholder.
    fn placeholder() -> Self {
        Self {
            image: None,
            image_cells: Vec2::ONE,
        }
    }

    /// The Material that draws this look over the square of `extent` cells whose lower-left
    /// corner is `origin`, masked by `coverage`.
    pub(crate) fn material(
        &self,
        origin: Vec2,
        extent: Vec2,
        coverage: Handle<Image>,
    ) -> TerrainMaterial {
        TerrainMaterial {
            params: Params {
                image_cells: self.image_cells,
                origin,
                extent,
                flat: if self.image.is_some() { 0.0 } else { 1.0 },
                colour: LinearRgba::from(PLACEHOLDER).to_vec4(),
            },
            image: self.image.clone(),
            coverage,
        }
    }
}

/// Where a Terrain's image stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImageState {
    /// The image is loading; the placeholder stands in.
    Loading,
    /// The image is loaded and shown.
    Shown,
    /// There is no image to show: Missing, or failed to load.
    Placeholder,
}

/// One coverage tile as drawn.
struct TileDrawn {
    /// The quad's entity.
    entity: Entity,
    /// The revision of the tile's pixels uploaded.
    revision: u64,
    /// The quad's Material.
    material: Handle<TerrainMaterial>,
    /// The coverage sampled: the pixels uploaded, or the tile's image on the GPU.
    coverage: Handle<Image>,
    /// The image on the GPU the coverage is, for a tile rasterized there.
    image: Option<GpuTile>,
}

/// One Terrain as drawn.
struct TerrainDrawn {
    /// The asset path of its image, or `None` when it has none.
    path: Option<AssetPath<'static>>,
    /// The image, once asked for.
    image: Option<Handle<Image>>,
    /// Where the image stands.
    state: ImageState,
    /// One repetition of the image, in Grid cells.
    image_cells: Vec2,
    /// Its depth in the stacking order.
    depth: f32,
    /// The look its tiles' Materials have.
    look: Look,
    /// The band its tiles are at.
    band: u32,
    /// Its coverage tiles as drawn, by position at the band.
    tiles: BTreeMap<TileKey, TileDrawn>,
}

impl TerrainDrawn {
    /// How the Terrain looks now.
    fn look(&self) -> Look {
        match (&self.image, self.state) {
            (Some(image), ImageState::Shown) => Look {
                image: Some(image.clone()),
                image_cells: self.image_cells,
            },
            _ => Look::placeholder(),
        }
    }
}

/// The Terrains as drawn in the viewport, by Element, and the quad every coverage is drawn on.
#[derive(Resource, Default)]
pub(crate) struct TerrainDrawings {
    /// The Terrains, by Element.
    terrains: BTreeMap<Entity, TerrainDrawn>,
    /// A square of one cell a side, scaled to the square each quad covers.
    quad: Option<Handle<Mesh>>,
}

impl TerrainDrawings {
    /// Whether some Terrain's image is still loading.
    pub(crate) fn loading(&self) -> bool {
        self.terrains
            .values()
            .any(|terrain| terrain.state == ImageState::Loading)
    }

    /// How a Terrain looks and its depth, or `None` for one not drawn.
    pub(crate) fn look_of(&self, element: Entity) -> Option<(Look, f32)> {
        self.terrains
            .get(&element)
            .map(|terrain| (terrain.look.clone(), terrain.depth))
    }

    /// The square of one cell a side every coverage is drawn on, made the first time.
    pub(crate) fn quad(&mut self, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
        self.quad
            .get_or_insert_with(|| meshes.add(unit_square()))
            .clone()
    }
}

/// A square of one cell a side centred on the origin, its texture coordinates running from the
/// top-left corner, as a coverage's rows do.
fn unit_square() -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [-0.5, 0.5, 0.0],
            [0.5, 0.5, 0.0],
            [0.5, -0.5, 0.0],
            [-0.5, -0.5, 0.0],
        ],
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
    )
    .with_inserted_indices(Indices::U32(vec![0, 3, 2, 0, 2, 1]))
}

/// Marks the quad of one coverage tile.
#[derive(Component)]
pub(crate) struct TerrainTile;

/// The assets Terrains are drawn with.
#[derive(SystemParam)]
pub(crate) struct TerrainAssets<'w> {
    /// The coverages, one image each.
    pub(crate) images: ResMut<'w, Assets<Image>>,
    /// The Materials, one per quad.
    pub(crate) materials: ResMut<'w, Assets<TerrainMaterial>>,
    /// The quad.
    pub(crate) meshes: ResMut<'w, Assets<Mesh>>,
}

/// A coverage of `side` texels a side as an image to sample: one byte a texel, rows from the
/// top, on the GPU only.
pub(crate) fn coverage_image(pixels: Vec<u8>, side: u32, sampler: ImageSampler) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: side,
            height: side,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::R8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = sampler;
    image
}

/// The transform of a quad covering the square of `extent` cells whose lower-left corner is
/// `origin`, at `depth`.
pub(crate) fn quad_transform(origin: Vec2, extent: Vec2, depth: f32) -> Transform {
    let centre = origin + extent / 2.0;
    Transform {
        translation: Vec3::new(centre.x, centre.y, depth),
        scale: Vec3::new(extent.x, extent.y, 1.0),
        ..Transform::default()
    }
}

/// Whether anything the Terrains' quads depend on changed since they were last brought in step,
/// or an image is still loading.
#[expect(
    clippy::type_complexity,
    reason = "a Bevy query filter is spelled out by the components it watches"
)]
pub(crate) fn terrains_changed(
    terrains: Query<(), Or<(Changed<Terrain>, Changed<TerrainCoverage>)>>,
    orders: Query<
        (),
        (
            Changed<Children>,
            Or<(With<Project>, With<Level>, With<ModelLayer>)>,
        ),
    >,
    tables: Query<(), Or<(Changed<ResolutionTable>, Changed<AssetReferences>)>>,
    drawings: Res<TerrainDrawings>,
    mut removed: RemovedComponents<TerrainCoverage>,
) -> bool {
    let removed = removed.read().count();
    !terrains.is_empty()
        || !orders.is_empty()
        || !tables.is_empty()
        || removed > 0
        || drawings.loading()
}

/// The parts of the model the Terrains are drawn from.
#[derive(SystemParam)]
pub(crate) struct Model<'w, 's> {
    /// The stacking order.
    stacking: Stacking<'w, 's>,
    /// How each known kind is drawn.
    kinds: Option<Res<'w, ElementKindRegistry>>,
    /// Each Project's resolution table, Asset References, and Grid.
    projects: Query<
        'w,
        's,
        (
            &'static ResolutionTable,
            &'static AssetReferences,
            Option<&'static Grid>,
        ),
        With<Project>,
    >,
    /// Every Terrain, and its coverage once derived.
    terrains: Query<
        'w,
        's,
        (
            &'static Element,
            &'static Terrain,
            Option<&'static TerrainCoverage>,
        ),
    >,
}

/// Brings the Terrains' quads in step with the model: for every Element drawn as a painted
/// surface, one quad per tile of its coverage at its depth in the stacking order, its coverage
/// uploaded again when its revision changes, drawn with its image once loaded and the flat
/// placeholder while it loads, is Missing, or failed; the quads of tiles and Terrains that are
/// gone are removed. A Terrain whose coverage is not derived yet is not drawn that frame.
pub(crate) fn sync_terrains(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    model: Model,
    mut drawings: ResMut<TerrainDrawings>,
    mut assets: TerrainAssets,
    mut transforms: Query<&mut Transform, With<TerrainTile>>,
) {
    let quad = drawings.quad(&mut assets.meshes);
    let mut unseen: BTreeMap<Entity, TerrainDrawn> = std::mem::take(&mut drawings.terrains);
    for stacked in model.stacking.in_order() {
        let (Ok((resolutions, references, grid)), Ok((element, terrain, coverage))) = (
            model.projects.get(stacked.project),
            model.terrains.get(stacked.element),
        ) else {
            continue;
        };
        if drawn_as(model.kinds.as_deref(), element) != Some(DrawnAs::PaintedSurface) {
            continue;
        }
        let path = match resolutions.get(terrain.image) {
            Some(Resolution::Resolved { folder, place }) => Some(asset_path(folder, place)),
            Some(Resolution::Missing(_)) | None => None,
        };
        #[expect(
            clippy::cast_precision_loss,
            reason = "pixels per cell and an image's pixels are far below where f32 loses whole \
                      numbers"
        )]
        let image_cells = references
            .get(terrain.image)
            .and_then(|reference| reference.pixel_size)
            .map_or(Vec2::ONE, |pixels| {
                pixels.as_vec2() / grid.copied().unwrap_or_default().pixels_per_cell as f32
            });
        let mut drawn = unseen
            .remove(&stacked.element)
            .unwrap_or_else(|| TerrainDrawn {
                path: None,
                image: None,
                state: ImageState::Placeholder,
                image_cells,
                depth: stacked.depth,
                look: Look::placeholder(),
                band: COVERAGE_PIXELS_PER_CELL,
                tiles: BTreeMap::new(),
            });
        if drawn.path != path {
            drawn.image = path.as_ref().map(|path| asset_server.load::<Image>(path));
            drawn.state = if path.is_some() {
                ImageState::Loading
            } else {
                ImageState::Placeholder
            };
            drawn.path = path;
        }
        drawn.image_cells = image_cells;
        if drawn.state == ImageState::Loading {
            settle(&mut drawn, &asset_server);
        }
        let look = drawn.look();
        let restyled = look != drawn.look;
        drawn.look = look;
        let moved = drawn.depth.total_cmp(&stacked.depth).is_ne();
        drawn.depth = stacked.depth;
        let empty = TerrainCoverage::default();
        sync_tiles(
            &mut drawn,
            coverage.unwrap_or(&empty),
            Restyle { restyled, moved },
            &quad,
            &mut commands,
            &mut assets,
            &mut transforms,
        );
        drawings.terrains.insert(stacked.element, drawn);
    }
    for gone in unseen.into_values() {
        for tile in gone.tiles.into_values() {
            commands.entity(tile.entity).despawn();
        }
    }
}

/// What changed about a Terrain as a whole since its tiles were last brought in step.
#[derive(Debug, Clone, Copy)]
struct Restyle {
    /// Its look: the tiles' Materials are set again.
    restyled: bool,
    /// Its depth: the tiles' quads are moved.
    moved: bool,
}

/// Brings one Terrain's quads in step with its coverage: a tile on the GPU is drawn from its
/// image as it is, with no upload, its Material given the image only when the tile names
/// another; a tile on the CPU whose revision changed is uploaded again and its Material marked
/// changed; a new tile gets a quad of its band's size at its corner, and the quads of tiles that
/// are gone, every one when the band changed, are removed.
fn sync_tiles(
    drawn: &mut TerrainDrawn,
    coverage: &TerrainCoverage,
    change: Restyle,
    quad: &Handle<Mesh>,
    commands: &mut Commands,
    assets: &mut TerrainAssets,
    transforms: &mut Query<&mut Transform, With<TerrainTile>>,
) {
    let mut tiles = std::mem::take(&mut drawn.tiles);
    if drawn.band != coverage.band {
        for gone in std::mem::take(&mut tiles).into_values() {
            commands.entity(gone.entity).despawn();
        }
        drawn.band = coverage.band;
    }
    let extent = Vec2::splat(tile_cells(coverage.band));
    for (key, tile) in &coverage.tiles {
        let origin = key.corner_at(coverage.band);
        if let Some(mut kept) = tiles.remove(key) {
            let shown = show(&mut kept, tile, assets);
            if shown == Shown::Gone {
                commands.entity(kept.entity).despawn();
                continue;
            }
            if (shown == Shown::Rebound || change.restyled)
                && let Some(material) = assets.materials.get_mut(&kept.material)
            {
                *material.into_inner() = drawn.look.material(origin, extent, kept.coverage.clone());
            } else if shown == Shown::Replaced
                && let Some(material) = assets.materials.get_mut(&kept.material)
            {
                // The Material's bind group holds the coverage texture it was prepared with, so
                // a replaced coverage shows only once the Material is marked changed too.
                material.into_inner();
            }
            if change.moved
                && let Ok(mut transform) = transforms.get_mut(kept.entity)
            {
                *transform = quad_transform(origin, extent, drawn.depth);
            }
            drawn.tiles.insert(*key, kept);
        } else {
            let (coverage, image) = match &tile.content {
                TileContent::Gpu(image) => match gpu_image(&mut assets.images, *image) {
                    Some(coverage) => (coverage, Some(*image)),
                    None => continue,
                },
                TileContent::Pixels(pixels) => (
                    assets.images.add(coverage_image(
                        pixels.to_vec(),
                        COVERAGE_TILE_PIXELS,
                        ImageSampler::linear(),
                    )),
                    None,
                ),
            };
            let material =
                assets
                    .materials
                    .add(drawn.look.material(origin, extent, coverage.clone()));
            let entity = commands
                .spawn((
                    Mesh2d(quad.clone()),
                    MeshMaterial2d(material.clone()),
                    quad_transform(origin, extent, drawn.depth),
                    RenderLayers::layer(VIEWPORT_LAYER),
                    TerrainTile,
                ))
                .id();
            drawn.tiles.insert(
                *key,
                TileDrawn {
                    entity,
                    revision: tile.revision,
                    material,
                    coverage,
                    image,
                },
            );
        }
    }
    for gone in tiles.into_values() {
        commands.entity(gone.entity).despawn();
    }
}

/// What showing a tile's new state did to the coverage a quad samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shown {
    /// Nothing: the same pixels, or the same image on the GPU, drawn into or not.
    Kept,
    /// The pixels uploaded again into the same image.
    Replaced,
    /// Another image: the Material needs it.
    Rebound,
    /// The tile's image on the GPU is gone, so the tile cannot be drawn.
    Gone,
}

/// Brings the coverage a drawn tile samples in step with the tile.
fn show(kept: &mut TileDrawn, tile: &CoverageTile, assets: &mut TerrainAssets) -> Shown {
    let shown = match (&tile.content, kept.image) {
        (TileContent::Gpu(image), Some(before)) if *image == before => Shown::Kept,
        (TileContent::Gpu(image), _) => match gpu_image(&mut assets.images, *image) {
            Some(coverage) => {
                kept.coverage = coverage;
                kept.image = Some(*image);
                Shown::Rebound
            }
            None => Shown::Gone,
        },
        (TileContent::Pixels(pixels), Some(_)) => {
            kept.coverage = assets.images.add(coverage_image(
                pixels.to_vec(),
                COVERAGE_TILE_PIXELS,
                ImageSampler::linear(),
            ));
            kept.image = None;
            Shown::Rebound
        }
        (TileContent::Pixels(pixels), None) if kept.revision != tile.revision => {
            let image = coverage_image(
                pixels.to_vec(),
                COVERAGE_TILE_PIXELS,
                ImageSampler::linear(),
            );
            if assets.images.insert(&kept.coverage, image).is_err() {
                log::warn!("the coverage of a Terrain could not be replaced");
            }
            Shown::Replaced
        }
        (TileContent::Pixels(_), None) => Shown::Kept,
    };
    kept.revision = tile.revision;
    shown
}

/// The image a tile rasterized on the GPU names, held while it is drawn, or `None` once the paint
/// Engine let it go.
fn gpu_image(images: &mut Assets<Image>, image: GpuTile) -> Option<Handle<Image>> {
    let found = images.get_strong_handle(AssetId::from(AssetIndex::from_bits(image.index_bits())));
    if found.is_none() {
        log::debug!("a Terrain's tile on the GPU is gone before it was drawn");
    }
    found
}

/// Settles a Terrain whose image was loading: a loaded image is shown, a failed one gives way to
/// the placeholder, and the editor keeps running either way.
fn settle(drawn: &mut TerrainDrawn, asset_server: &AssetServer) {
    let Some(image) = drawn.image.as_ref() else {
        drawn.state = ImageState::Placeholder;
        return;
    };
    match asset_server.get_load_state(image.id()) {
        None | Some(LoadState::NotLoaded | LoadState::Loading) => {}
        Some(LoadState::Loaded) => drawn.state = ImageState::Shown,
        Some(LoadState::Failed(error)) => {
            log::warn!(
                "the image {} of a Terrain could not be loaded, so a placeholder stands in: \
                 {error}",
                drawn
                    .path
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default()
            );
            drawn.state = ImageState::Placeholder;
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::missing_panics_doc,
        reason = "a test stops at the first thing that is not as expected"
    )]
    use super::SHADER_SOURCE;
    use naga::valid::{Capabilities, ValidationFlags, Validator};

    /// The masked tiled image Shader parses and validates as WGSL, so a mistake in it shows
    /// without a GPU, before Bevy compiles it at the first frame that draws a Terrain.
    #[test]
    fn the_shader_is_valid_wgsl() {
        let module = naga::front::wgsl::parse_str(SHADER_SOURCE)
            .unwrap_or_else(|error| panic!("{}", error.emit_to_string(SHADER_SOURCE)));
        Validator::new(ValidationFlags::all(), Capabilities::default())
            .validate(&module)
            .unwrap_or_else(|error| panic!("{}", error.emit_to_string(SHADER_SOURCE)));
    }
}
