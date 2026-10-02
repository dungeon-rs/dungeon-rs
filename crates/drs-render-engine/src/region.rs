//! `RenderRegion`: a rectangle of the Level rendered offscreen at a chosen resolution and read
//! back from the GPU, tile by tile, for the Export.
//!
//! One offscreen camera draws into a square target texture of the tile size, through the same
//! sprites and depths as the viewport, over an opaque black clear colour. Rendering and reading
//! back take frames, so the operation is a request and a poll: [`request_region`] points the
//! camera at a region and [`take_region`] yields its pixels once they have come back, which is
//! two or three frames later. A region is captured only once every image a sprite is loading has
//! loaded or failed, and never in the frame the camera was spawned in, whose render is not yet
//! the camera's own. [`release_regions`] removes the camera when the Export is done.
//!
//! The readback is the Engine's own: the render world takes the region at extraction, once its
//! target exists on the GPU, copies the target into a staging buffer after the frame is drawn,
//! maps it, and hands the bytes to the main world at the next extraction. No event or observer
//! is involved, so the Engine is called and never notified.
//!
//! The sprites keep the sampler they are drawn with in the viewport, the editor's default of
//! linear filtering, so an image scales between the Grid's pixels per cell and the region's by
//! linear interpolation, in the Export as on screen.
//!
//! A Terrain is drawn from the coverage the request hands in, computed for the region at its
//! resolution, one quad of the region's size per Terrain that only the export camera sees, in
//! place of the viewport's coverage tiles, which it does not see. The coverage's texels fall on
//! the region's pixels one to one.

use crate::projection::DEPTH;
use crate::props::Loading;
use crate::terrain::{
    EXPORT_LAYER, TerrainAssets, TerrainDrawings, coverage_image, quad_transform,
};
use bevy_asset::{Assets, Handle};
use bevy_camera::visibility::RenderLayers;
use bevy_camera::{
    Camera, Camera2d, ClearColorConfig, OrthographicProjection, Projection, RenderTarget,
    ScalingMode,
};
use bevy_color::Color;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::With;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{Commands, Query, Res, ResMut};
use bevy_ecs::world::World;
use bevy_image::{Image, ImageSampler};
use bevy_math::{Vec2, Vec3};
use bevy_mesh::Mesh2d;
use bevy_render::MainWorld;
use bevy_render::render_asset::RenderAssets;
use bevy_render::render_resource::{
    Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Extent3d, MapMode, PollType,
    TexelCopyBufferInfo, TexelCopyBufferLayout, Texture, TextureFormat, TextureUsages,
};
use bevy_render::renderer::{RenderDevice, RenderQueue};
use bevy_render::texture::GpuImage;
use bevy_sprite_render::MeshMaterial2d;
use bevy_transform::components::Transform;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

/// Bytes per RGBA8 texel.
const BYTES_PER_PIXEL: u32 = 4;

/// The largest tile the Engine renders: the smallest texture size every supported GPU allows.
pub const MOST_TILE_PIXELS: u32 = 8192;

/// The export camera draws after the viewport's, which draws at order zero.
const EXPORT_ORDER: isize = 1;

/// What can go wrong while rendering a region.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    /// No renderer runs in this `World`, so nothing can be drawn offscreen.
    #[error("no renderer is running")]
    NoRenderer,
    /// The tile size is zero or larger than a texture may be.
    #[error("a tile of {0} pixels a side cannot be rendered; the most is {MOST_TILE_PIXELS}")]
    BadTileSize(u32),
    /// The resolution is zero.
    #[error("a resolution of zero pixels per cell cannot be rendered")]
    ZeroResolution,
    /// A region was requested while the previous one has not been captured yet.
    #[error("a region is already waiting to be captured")]
    RegionPending,
    /// The GPU did not hand the region back.
    #[error("the region could not be read back from the GPU: {0}")]
    Readback(String),
}

/// A region handed out by [`request_region`], redeemed through [`take_region`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RegionRequest(u64);

/// The pixels of a rendered region: a square of `size` texels a side in RGBA8, rows from top to
/// bottom, every texel opaque.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionPixels {
    /// The side of the square in texels.
    pub size: u32,
    /// `size * size * 4` bytes.
    pub rgba: Vec<u8>,
}

/// The coverage of one Terrain over a requested region, computed at the region's resolution:
/// one byte a pixel, rows from the region's top, as many as the region has pixels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionCoverage {
    /// The Terrain's Element.
    pub terrain: Entity,
    /// The coverage, 255 where it is full.
    pub pixels: Vec<u8>,
}

/// The coverages handed in with the pending region, to be drawn before it is captured.
struct RegionTerrains {
    /// The lower-left corner of the region, in cells.
    bottom_left: Vec2,
    /// The side of the region in pixels.
    size: u32,
    /// The side of the region in cells.
    cells: f32,
    /// The coverages, one per Terrain that covers something there.
    coverages: Vec<RegionCoverage>,
}

/// Marks the offscreen camera the regions are drawn with.
#[derive(Component)]
struct ExportCamera;

/// The offscreen camera as spawned.
struct CameraRecord {
    /// The camera entity.
    entity: Entity,
    /// The texture it draws into.
    target: Handle<Image>,
    /// The side of the target in pixels.
    tile_size: u32,
    /// The resolution its projection is set for.
    pixels_per_cell: u32,
}

/// A region the camera is pointed at, waiting for its frame to be captured.
struct Pending {
    /// The request it answers.
    request: RegionRequest,
    /// The texture to read back.
    target: Handle<Image>,
    /// The side of the texture in pixels.
    size: u32,
}

/// The main world's side of the offscreen rendering.
#[derive(Resource, Default)]
pub(crate) struct Offscreen {
    /// The camera, once spawned.
    camera: Option<CameraRecord>,
    /// Whether the camera was spawned this frame, so this frame's render is not yet its own.
    fresh: bool,
    /// The request handed out last.
    last: u64,
    /// The region waiting to be captured, if any.
    pending: Option<Pending>,
    /// Whether the pending region may be captured at the end of this frame.
    capture: bool,
    /// The requests handed out and neither taken nor released.
    outstanding: BTreeSet<RegionRequest>,
    /// The regions that came back, by request.
    completed: BTreeMap<RegionRequest, Completed>,
    /// The coverages of the pending region, until they are drawn.
    terrains: Option<RegionTerrains>,
    /// The quads the coverages of the last region are drawn on.
    quads: Vec<Entity>,
}

/// A region that came back from the GPU.
struct Completed {
    /// The side of the region in pixels.
    size: u32,
    /// Its bytes with the rows still padded, or why it could not be read back.
    outcome: Result<Vec<u8>, String>,
}

/// `RenderRegion`, first half: points the offscreen camera at the square of the Level whose
/// lower-left corner is `bottom_left`, in cells, and whose side is `tile_size` pixels at
/// `pixels_per_cell`, and asks for its pixels, its Terrains drawn from `coverages`, computed
/// over that square at that resolution; a Terrain without one is not drawn there. The coverages
/// are taken out of `coverages` once the request is accepted, and left there when it is
/// refused, for the request to be made again.
///
/// The pixels come back through [`take_region`] a few frames later. One region is captured per
/// frame: a request is refused while the previous region still waits for its frame to be drawn,
/// which it does for a frame, or longer while an image is loading, and is accepted once that
/// frame is on its way, before its pixels are back. The pixels come back in the order of the
/// requests, so an Export may keep a few requests in flight.
///
/// # Errors
///
/// [`RenderError::NoRenderer`] without a renderer, [`RenderError::BadTileSize`] or
/// [`RenderError::ZeroResolution`] for sizes that cannot be drawn, or
/// [`RenderError::RegionPending`] while the previous region waits to be captured.
pub fn request_region(
    world: &mut World,
    bottom_left: Vec2,
    pixels_per_cell: u32,
    tile_size: u32,
    coverages: &mut Vec<RegionCoverage>,
) -> Result<RegionRequest, RenderError> {
    if tile_size == 0 || tile_size > MOST_TILE_PIXELS {
        return Err(RenderError::BadTileSize(tile_size));
    }
    if pixels_per_cell == 0 {
        return Err(RenderError::ZeroResolution);
    }
    let offscreen = world
        .get_resource::<Offscreen>()
        .ok_or(RenderError::NoRenderer)?;
    if offscreen.pending.is_some() {
        return Err(RenderError::RegionPending);
    }
    let fits = offscreen.camera.as_ref().is_some_and(|camera| {
        camera.tile_size == tile_size && camera.pixels_per_cell == pixels_per_cell
    });
    if !fits {
        release_camera(world);
        spawn_camera(world, pixels_per_cell, tile_size)?;
    }
    let (entity, request) = {
        let mut offscreen = world
            .get_resource_mut::<Offscreen>()
            .ok_or(RenderError::NoRenderer)?;
        let Some(camera) = offscreen.camera.as_ref() else {
            return Err(RenderError::NoRenderer);
        };
        let (entity, target) = (camera.entity, camera.target.clone());
        offscreen.last += 1;
        let request = RegionRequest(offscreen.last);
        offscreen.outstanding.insert(request);
        offscreen.pending = Some(Pending {
            request,
            target,
            size: tile_size,
        });
        offscreen.capture = false;
        #[expect(
            clippy::cast_precision_loss,
            reason = "a tile is at most 8192 pixels, far below where f32 loses whole numbers"
        )]
        let cells = tile_size as f32 / pixels_per_cell as f32;
        offscreen.terrains = Some(RegionTerrains {
            bottom_left,
            size: tile_size,
            cells,
            coverages: std::mem::take(coverages),
        });
        (entity, request)
    };
    if let Some(mut transform) = world.get_mut::<Transform>(entity) {
        transform.translation = Vec3::new(bottom_left.x, bottom_left.y, 0.0);
    }
    Ok(request)
}

/// `RenderRegion`, second half: the pixels of a requested region, once they have come back.
///
/// Returns `Ok(None)` while the region is still being rendered or read back. The pixels are
/// handed out once; a request that was taken or released yields `Ok(None)` for ever after.
///
/// # Errors
///
/// [`RenderError::NoRenderer`] without a renderer, or [`RenderError::Readback`] when the GPU
/// could not hand the region back.
pub fn take_region(
    world: &mut World,
    request: RegionRequest,
) -> Result<Option<RegionPixels>, RenderError> {
    let mut offscreen = world
        .get_resource_mut::<Offscreen>()
        .ok_or(RenderError::NoRenderer)?;
    let Some(completed) = offscreen.completed.remove(&request) else {
        return Ok(None);
    };
    offscreen.outstanding.remove(&request);
    let padded = completed.outcome.map_err(RenderError::Readback)?;
    Ok(Some(RegionPixels {
        size: completed.size,
        rgba: unpad(&padded, completed.size),
    }))
}

/// Removes the offscreen camera and forgets every region requested so far.
pub fn release_regions(world: &mut World) {
    release_camera(world);
    let quads = world
        .get_resource_mut::<Offscreen>()
        .map(|mut offscreen| {
            offscreen.pending = None;
            offscreen.capture = false;
            offscreen.outstanding.clear();
            offscreen.completed.clear();
            offscreen.terrains = None;
            std::mem::take(&mut offscreen.quads)
        })
        .unwrap_or_default();
    for quad in quads {
        world.despawn(quad);
    }
}

/// Spawns the offscreen camera with a target of `tile_size` pixels a side and a projection
/// showing `tile_size / pixels_per_cell` cells, its translation the lower-left corner shown.
///
/// The camera is spawned after the viewport's camera, so a UI toolkit that attaches itself to
/// the first camera spawned leaves it alone.
///
/// # Errors
///
/// [`RenderError::NoRenderer`] when the `World` holds no images or no offscreen state.
fn spawn_camera(
    world: &mut World,
    pixels_per_cell: u32,
    tile_size: u32,
) -> Result<(), RenderError> {
    let mut target =
        Image::new_target_texture(tile_size, tile_size, TextureFormat::Rgba8UnormSrgb, None);
    target.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    let target = world
        .get_resource_mut::<Assets<Image>>()
        .ok_or(RenderError::NoRenderer)?
        .add(target);
    #[expect(
        clippy::cast_precision_loss,
        reason = "a tile is at most 8192 pixels, far below where f32 loses whole numbers"
    )]
    let cells = tile_size as f32 / pixels_per_cell as f32;
    let entity = world
        .spawn((
            Camera2d,
            Camera {
                order: EXPORT_ORDER,
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..Camera::default()
            },
            RenderTarget::Image(target.clone().into()),
            RenderLayers::from_layers(&[0, EXPORT_LAYER]),
            Projection::Orthographic(OrthographicProjection {
                near: -DEPTH,
                far: DEPTH,
                viewport_origin: Vec2::ZERO,
                scaling_mode: ScalingMode::Fixed {
                    width: cells,
                    height: cells,
                },
                ..OrthographicProjection::default_2d()
            }),
            Transform::default(),
            ExportCamera,
        ))
        .id();
    let mut offscreen = world
        .get_resource_mut::<Offscreen>()
        .ok_or(RenderError::NoRenderer)?;
    offscreen.camera = Some(CameraRecord {
        entity,
        target,
        tile_size,
        pixels_per_cell,
    });
    offscreen.fresh = true;
    Ok(())
}

/// Despawns the offscreen camera, if any, which also frees its target.
fn release_camera(world: &mut World) {
    let camera = world
        .get_resource_mut::<Offscreen>()
        .and_then(|mut offscreen| offscreen.camera.take());
    if let Some(camera) = camera {
        world.despawn(camera.entity);
    }
}

/// Whether the coverages of the pending region wait to be drawn and no Terrain's image is
/// loading, so they can be.
pub(crate) fn region_terrains_ready(
    offscreen: Res<Offscreen>,
    drawings: Res<TerrainDrawings>,
) -> bool {
    offscreen.terrains.is_some() && !drawings.loading()
}

/// Draws the coverages of the pending region: one quad of the region's size per Terrain, at
/// the Terrain's depth and with its look, that only the export camera sees, in place of the
/// last region's quads.
///
/// The region is then captured a frame later, so that the quads are in place on the GPU.
pub(crate) fn draw_region_terrains(
    mut commands: Commands,
    mut offscreen: ResMut<Offscreen>,
    mut drawings: ResMut<TerrainDrawings>,
    mut assets: TerrainAssets,
) {
    let Some(terrains) = offscreen.terrains.take() else {
        return;
    };
    for quad in std::mem::take(&mut offscreen.quads) {
        commands.entity(quad).despawn();
    }
    let mesh = drawings.quad(&mut assets.meshes);
    let extent = Vec2::splat(terrains.cells);
    for coverage in terrains.coverages {
        let Some((look, depth)) = drawings.look_of(coverage.terrain) else {
            continue;
        };
        if coverage.pixels.len() != (terrains.size as usize).pow(2) {
            log::warn!("the coverage of a Terrain does not fit the region, so it is not drawn");
            continue;
        }
        let image = assets.images.add(coverage_image(
            coverage.pixels,
            terrains.size,
            ImageSampler::nearest(),
        ));
        let material = assets
            .materials
            .add(look.material(terrains.bottom_left, extent, image));
        let quad = commands
            .spawn((
                Mesh2d(mesh.clone()),
                MeshMaterial2d(material),
                quad_transform(terrains.bottom_left, extent, depth),
                RenderLayers::layer(EXPORT_LAYER),
            ))
            .id();
        offscreen.quads.push(quad);
    }
    if !offscreen.quads.is_empty() {
        offscreen.fresh = true;
    }
}

/// Decides at the end of the frame whether the pending region may be captured: not in the frame
/// the camera was spawned in or its Terrains were drawn in, only once its Terrains are drawn,
/// and only once no sprite or Terrain is still loading its image.
pub(crate) fn gate_regions(
    mut offscreen: ResMut<Offscreen>,
    loading: Query<(), With<Loading>>,
    drawings: Res<TerrainDrawings>,
) {
    let fresh = std::mem::take(&mut offscreen.fresh);
    let capture = offscreen.pending.is_some()
        && !fresh
        && loading.is_empty()
        && offscreen.terrains.is_none()
        && !drawings.loading();
    if offscreen.capture != capture {
        offscreen.capture = capture;
    }
}

/// Copies `size` rows of `size * 4` bytes out of a readback whose rows are padded to the GPU's
/// copy alignment.
fn unpad(padded: &[u8], size: u32) -> Vec<u8> {
    let row = (size * BYTES_PER_PIXEL) as usize;
    let padded_row = padded_row_bytes(size) as usize;
    if padded_row == row {
        return padded.to_vec();
    }
    let mut rgba = Vec::with_capacity(row * size as usize);
    for y in 0..size as usize {
        let start = y * padded_row;
        rgba.extend_from_slice(&padded[start..start + row]);
    }
    rgba
}

/// The bytes one row of a `size`-texel-wide RGBA8 texture takes in a copy buffer.
fn padded_row_bytes(size: u32) -> u32 {
    u32::try_from(RenderDevice::align_copy_bytes_per_row(
        (size * BYTES_PER_PIXEL) as usize,
    ))
    .unwrap_or(u32::MAX)
}

// --- The render world's half ---------------------------------------------------------------------

/// A region whose frame is being drawn, with a staging buffer ready for the copy once it is.
struct Staged {
    /// The request it answers.
    request: RegionRequest,
    /// The texture drawn into.
    texture: Texture,
    /// The buffer the texture is copied into.
    buffer: Buffer,
    /// The side of the texture in pixels.
    size: u32,
}

/// Where a mapping callback leaves the bytes it read, or why it could not.
type Slot = Arc<Mutex<Option<Result<Vec<u8>, String>>>>;

/// A region whose buffer is being mapped; the bytes arrive in the slot.
struct Mapped {
    /// The request it answers.
    request: RegionRequest,
    /// The side of the region in pixels.
    size: u32,
    /// Where the mapped bytes, or the failure, arrive.
    slot: Slot,
}

/// The render world's side of the offscreen rendering.
#[derive(Resource, Default)]
pub(crate) struct Readbacks {
    /// Regions taken from the main world this frame, copied once the frame is drawn.
    staged: Vec<Staged>,
    /// Regions whose buffers are being mapped.
    mapped: Vec<Mapped>,
}

/// Hands back the regions that have arrived and takes the region the main world lets be
/// captured this frame, giving it a staging buffer.
///
/// A region is taken only once its target exists on the GPU; until then the camera has drawn
/// nothing into it, so the region stays pending, the camera stays where it is, and no later
/// request can move it before this one is captured.
pub(crate) fn extract_regions(
    mut main_world: ResMut<MainWorld>,
    mut readbacks: ResMut<Readbacks>,
    device: Res<RenderDevice>,
    images: Res<RenderAssets<GpuImage>>,
) {
    // Mapping callbacks fire when the device is polled; a non-waiting poll costs nothing.
    let _ = device.poll(PollType::Poll);
    let Some(mut offscreen) = main_world.get_resource_mut::<Offscreen>() else {
        return;
    };
    let mut arrived = Vec::new();
    readbacks.mapped.retain(|mapped| {
        let outcome = mapped.slot.lock().ok().and_then(|mut slot| slot.take());
        match outcome {
            Some(outcome) => {
                arrived.push((
                    mapped.request,
                    Completed {
                        size: mapped.size,
                        outcome,
                    },
                ));
                false
            }
            None => true,
        }
    });
    for (request, completed) in arrived {
        if offscreen.outstanding.contains(&request) {
            offscreen.completed.insert(request, completed);
        }
    }
    if !offscreen.capture {
        return;
    }
    offscreen.capture = false;
    let Some(pending) = offscreen.pending.as_ref() else {
        return;
    };
    let Some(image) = images.get(&pending.target) else {
        return;
    };
    let buffer = device.create_buffer(&BufferDescriptor {
        label: Some("export region readback"),
        size: u64::from(padded_row_bytes(pending.size)) * u64::from(pending.size),
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    readbacks.staged.push(Staged {
        request: pending.request,
        texture: image.texture.clone(),
        buffer,
        size: pending.size,
    });
    offscreen.pending = None;
}

/// Copies each staged region's texture into its buffer after the frame has been drawn, and
/// starts mapping the buffer.
pub(crate) fn copy_regions(
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut readbacks: ResMut<Readbacks>,
) {
    let staged = std::mem::take(&mut readbacks.staged);
    if staged.is_empty() {
        return;
    }
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("export region copy"),
    });
    for region in &staged {
        encoder.copy_texture_to_buffer(
            region.texture.as_image_copy(),
            TexelCopyBufferInfo {
                buffer: &region.buffer,
                layout: TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row_bytes(region.size)),
                    rows_per_image: None,
                },
            },
            Extent3d {
                width: region.size,
                height: region.size,
                depth_or_array_layers: 1,
            },
        );
    }
    queue.submit([encoder.finish()]);
    for region in staged {
        let slot: Slot = Arc::default();
        let filled = Arc::clone(&slot);
        let buffer = region.buffer.clone();
        region
            .buffer
            .slice(..)
            .map_async(MapMode::Read, move |outcome| {
                let bytes = outcome.map_err(|error| error.to_string()).and_then(|()| {
                    let bytes = buffer
                        .get_mapped_range(..)
                        .map(|view| view.to_vec())
                        .map_err(|error| error.to_string());
                    buffer.unmap();
                    bytes
                });
                if let Ok(mut slot) = filled.lock() {
                    *slot = Some(bytes);
                }
            });
        readbacks.mapped.push(Mapped {
            request: region.request,
            size: region.size,
            slot,
        });
    }
}
