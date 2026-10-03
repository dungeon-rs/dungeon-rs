//! Rasterizing strokes on the GPU: the tiles as render-world images, the work the main world
//! hands over each frame, and the render-world pass that draws it into the tiles before any
//! camera renders the frame.

use bevy_app::{App, Plugin};
use bevy_asset::{AssetId, Assets, Handle, RenderAssetUsages, uuid_handle};
use bevy_ecs::resource::Resource;
use bevy_ecs::schedule::IntoScheduleConfigs;
use bevy_ecs::system::{Commands, Res, ResMut, SystemParam};
use bevy_image::{Image, ImageSampler};
use bevy_math::{IVec2, Vec2};
use bevy_render::render_asset::RenderAssets;
use bevy_render::render_resource::binding_types::{storage_buffer_read_only, uniform_buffer};
use bevy_render::render_resource::{
    BindGroup, BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries, BlendComponent,
    BlendFactor, BlendOperation, BlendState, CachedPipelineState, CachedRenderPipelineId,
    ColorTargetState, ColorWrites, DynamicUniformBuffer, Extent3d, FragmentState, LoadOp,
    Operations, PipelineCache, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipelineDescriptor, ShaderStages, ShaderType, StorageBuffer, StoreOp, TextureDimension,
    TextureFormat, TextureUsages, VertexState,
};
use bevy_render::renderer::{
    RenderContext, RenderDevice, RenderGraph, RenderGraphSystems, RenderQueue,
};
use bevy_render::texture::GpuImage;
use bevy_render::{ExtractSchedule, MainWorld, Render, RenderApp, RenderStartup, RenderSystems};
use bevy_shader::{Shader, ShaderCacheError};
use drs_model::{COVERAGE_TILE_PIXELS, GpuTile, Stroke, TileKey};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

/// The stroke Shader, a Bundled File compiled into the Engine.
const SHADER_SOURCE: &str = include_str!("shaders/stroke.wgsl");

/// The handle the stroke Shader is added under.
const SHADER: Handle<Shader> = uuid_handle!("74ff5a01-e33d-4b6c-8deb-0251b15a080c");

/// Rasterizes strokes into coverage tiles on the GPU, when the editor renders: in the render
/// world, before any camera renders a frame, it draws the work [`apply_stroke`](crate::apply_stroke)
/// handed over in that frame into the tiles' images. Without a renderer it does nothing, and
/// coverage is rasterized on the CPU.
pub struct PaintEnginePlugin;

impl Plugin for PaintEnginePlugin {
    fn build(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .init_resource::<Pending>()
            .add_systems(RenderStartup, queue_pipelines)
            .add_systems(ExtractSchedule, extract_jobs)
            .add_systems(
                Render,
                prepare_jobs.in_set(RenderSystems::PrepareBindGroups),
            )
            // Begin runs before the set the cameras render in, so every tile is complete before
            // anything samples it.
            .add_systems(
                RenderGraph,
                rasterize_jobs.in_set(RenderGraphSystems::Begin),
            );
        app.init_resource::<StrokeJobs>();
    }

    /// Adds the stroke Shader once every plugin is built, so the shader assets exist whichever
    /// order the plugins were added in. Without it no work is handed to the render world, which
    /// could never draw it, and coverage is rasterized on the CPU.
    fn finish(&self, app: &mut App) {
        if app.world().get_resource::<StrokeJobs>().is_none() {
            return;
        }
        let Some(mut shaders) = app.world_mut().get_resource_mut::<Assets<Shader>>() else {
            log::warn!(
                "there are no shader assets to add the stroke Shader to, so Terrain is \
                 rasterized on the CPU"
            );
            app.world_mut().remove_resource::<StrokeJobs>();
            return;
        };
        if shaders
            .insert(
                &SHADER,
                Shader::from_wgsl(SHADER_SOURCE, "drs-paint-engine/stroke.wgsl"),
            )
            .is_err()
        {
            log::warn!("the stroke Shader could not be added, so Terrain is rasterized on the CPU");
            app.world_mut().remove_resource::<StrokeJobs>();
        }
    }
}

/// A segment of a stroke's path as the stroke Shader reads it.
#[derive(ShaderType, Debug, Clone, Copy, PartialEq)]
pub(crate) struct SegmentData {
    /// Where it starts, in Grid cells.
    start: Vec2,
    /// Where it ends; the same point for a one-point path.
    end: Vec2,
    /// The Brush's radius, in Grid cells.
    radius: f32,
    /// How much of the radius shows the full strength.
    hardness: f32,
    /// How much of the Material the stroke shows where it shows the most.
    strength: f32,
    /// Fills the segment to a whole number of 16-byte rows.
    padding: f32,
}

/// The tile a pass draws into, as the stroke Shader reads it.
#[derive(ShaderType, Debug, Clone, Copy, PartialEq)]
struct TileData {
    /// The whole-pixel index of the tile's lower-left pixel along each axis.
    origin: IVec2,
    /// How many pixels one Grid cell spans.
    band: f32,
    /// Fills the tile to 16 bytes.
    padding: f32,
}

/// Consecutive strokes of one kind drawn into a tile in one draw: their segments, one quad each.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Run {
    /// Whether they erase.
    pub(crate) erase: bool,
    /// Their segments in the frame's segments.
    pub(crate) segments: Range<u32>,
}

/// One tile drawn into: cleared first, or drawn onto, and then each run in order.
#[derive(Debug, Clone)]
pub(crate) struct Pass {
    /// The tile's image.
    pub(crate) image: Handle<Image>,
    /// Where the tile is, at its band.
    pub(crate) key: TileKey,
    /// How many pixels one Grid cell spans in the tile.
    pub(crate) band: u32,
    /// Whether the tile is cleared before the runs are drawn.
    pub(crate) clear: bool,
    /// The runs, in the order the strokes were laid.
    pub(crate) runs: Vec<Run>,
}

/// The work rasterizing on the GPU has been handed in the frame and not yet taken to the render
/// world: the tiles to draw into, in order, and the segments their strokes are made of. The
/// render world takes it once a frame; a Terrain's cache adds to it through
/// [`StrokeRasterizer`].
#[derive(Resource, Debug, Default)]
pub struct StrokeJobs {
    /// The segments of every stroke the passes draw.
    segments: Vec<SegmentData>,
    /// The tiles to draw into, in the order they are drawn.
    passes: Vec<Pass>,
}

impl StrokeJobs {
    /// The tiles to draw into, in order.
    #[cfg(test)]
    pub(crate) fn passes(&self) -> &[Pass] {
        &self.passes
    }

    /// Hands over drawing into tiles: each of `drawn` names a tile, whether it is cleared, and
    /// the strokes of `strokes`, by number in order, drawn into it. The segments of the strokes
    /// named are laid out once, in the order of the strokes, so a run of consecutive strokes of
    /// one kind is one range of segments; a stroke between them that is of the other kind
    /// reaches none of the tile, so drawing its segments with the run changes no pixel.
    pub(crate) fn hand_over(&mut self, strokes: &[Stroke], drawn: Vec<Drawn>) {
        let named: BTreeSet<usize> = drawn
            .iter()
            .flat_map(|drawn| drawn.strokes.iter().copied())
            .collect();
        let mut laid: BTreeMap<usize, Range<u32>> = BTreeMap::new();
        for number in named {
            let Some(stroke) = strokes.get(number) else {
                continue;
            };
            let first = segment_count(&self.segments);
            self.segments.extend(segments(stroke));
            laid.insert(number, first..segment_count(&self.segments));
        }
        for drawn in drawn {
            let mut runs: Vec<Run> = Vec::new();
            for number in drawn.strokes {
                let (Some(stroke), Some(range)) = (strokes.get(number), laid.get(&number)) else {
                    continue;
                };
                match runs.last_mut() {
                    Some(run) if run.erase == stroke.erase => run.segments.end = range.end,
                    Some(_) | None => runs.push(Run {
                        erase: stroke.erase,
                        segments: range.clone(),
                    }),
                }
            }
            self.passes.push(Pass {
                image: drawn.image,
                key: drawn.key,
                band: drawn.band,
                clear: drawn.clear,
                runs,
            });
        }
    }
}

/// How many segments there are, as an index into them.
fn segment_count(segments: &[SegmentData]) -> u32 {
    u32::try_from(segments.len()).unwrap_or(u32::MAX)
}

/// The segments of a stroke's path as the Shader reads them: one from each point to the next, or
/// a single one of no length for a one-point path.
fn segments(stroke: &Stroke) -> Vec<SegmentData> {
    let segment = |start: Vec2, end: Vec2| SegmentData {
        start,
        end,
        radius: stroke.brush.radius(),
        hardness: stroke.brush.hardness,
        strength: stroke.brush.strength,
        padding: 0.0,
    };
    match stroke.points.as_slice() {
        [] => Vec::new(),
        [point] => vec![segment(*point, *point)],
        points => points
            .windows(2)
            .map(|pair| segment(pair[0], pair[1]))
            .collect(),
    }
}

/// A tile to draw into, as the cache plans it.
#[derive(Debug, Clone)]
pub(crate) struct Drawn {
    /// The tile's image.
    pub(crate) image: Handle<Image>,
    /// Where the tile is.
    pub(crate) key: TileKey,
    /// Its band.
    pub(crate) band: u32,
    /// Whether it is cleared first.
    pub(crate) clear: bool,
    /// The strokes drawn into it, by number, in order.
    pub(crate) strokes: Vec<usize>,
}

/// What rasterizes coverage tiles: the GPU when the editor renders, through the work handed to
/// [`PaintEnginePlugin`], and otherwise the CPU. A system that calls
/// [`apply_stroke`](crate::apply_stroke) takes it as a parameter.
#[derive(SystemParam)]
pub struct StrokeRasterizer<'w> {
    /// The work handed to the render world, present only when there is one.
    jobs: Option<ResMut<'w, StrokeJobs>>,
    /// The images the tiles on the GPU are.
    images: Option<ResMut<'w, Assets<Image>>>,
}

impl StrokeRasterizer<'_> {
    /// Whether there is a GPU to rasterize on.
    pub(crate) fn has_gpu(&self) -> bool {
        self.jobs.is_some() && self.images.is_some()
    }

    /// The GPU to rasterize on, or `None` to rasterize on the CPU.
    pub(crate) fn gpu(&mut self) -> Option<Gpu<'_>> {
        match (self.jobs.as_deref_mut(), self.images.as_deref_mut()) {
            (Some(jobs), Some(images)) => Some(Gpu { jobs, images }),
            (Some(_) | None, _) => None,
        }
    }
}

/// The GPU as the cache rasterizes on it: the work it hands over and the images it makes.
pub(crate) struct Gpu<'a> {
    /// The work handed over this frame.
    pub(crate) jobs: &'a mut StrokeJobs,
    /// The images the tiles are.
    pub(crate) images: &'a mut Assets<Image>,
}

impl Gpu<'_> {
    /// A new tile's image, on the GPU alone, and the identity the model names it by; `None`, with
    /// a warning, should the image have no asset index to name it by.
    pub(crate) fn new_tile(&mut self) -> Option<(Handle<Image>, GpuTile)> {
        let mut image = Image::new_uninit(
            Extent3d {
                width: COVERAGE_TILE_PIXELS,
                height: COVERAGE_TILE_PIXELS,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            TextureFormat::R8Unorm,
            RenderAssetUsages::RENDER_WORLD,
        );
        image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_DST
            | TextureUsages::RENDER_ATTACHMENT
            | TextureUsages::COPY_SRC;
        image.sampler = ImageSampler::linear();
        let handle = self.images.add(image);
        match handle.id() {
            AssetId::Index { index, .. } => {
                Some((handle, GpuTile::from_index_bits(index.to_bits())))
            }
            AssetId::Uuid { .. } => {
                log::warn!("a Terrain's tile image has no asset index to name it by");
                None
            }
        }
    }
}

/// The stroke pipeline in its two variants and the layout of its bind group.
#[derive(Resource)]
struct StrokePipelines {
    /// The bind group's layout: the tile, at a dynamic offset, and the segments.
    layout: BindGroupLayoutDescriptor,
    /// Strokes that paint: the target keeps the larger value.
    paint: CachedRenderPipelineId,
    /// Erases: the target keeps the smaller value.
    erase: CachedRenderPipelineId,
}

/// Queues the stroke pipeline's two variants.
fn queue_pipelines(mut commands: Commands, cache: Res<PipelineCache>) {
    let layout = BindGroupLayoutDescriptor::new(
        "drs-paint-engine strokes",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::VERTEX_FRAGMENT,
            (
                uniform_buffer::<TileData>(true),
                storage_buffer_read_only::<Vec<SegmentData>>(false),
            ),
        ),
    );
    let pipeline = |label: &'static str, fragment: &'static str, operation: BlendOperation| {
        let component = BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::One,
            operation,
        };
        RenderPipelineDescriptor {
            label: Some(label.into()),
            layout: vec![layout.clone()],
            vertex: VertexState {
                shader: SHADER,
                entry_point: Some("capsule".into()),
                ..VertexState::default()
            },
            fragment: Some(FragmentState {
                shader: SHADER,
                entry_point: Some(fragment.into()),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::R8Unorm,
                    blend: Some(BlendState {
                        color: component,
                        alpha: component,
                    }),
                    write_mask: ColorWrites::ALL,
                })],
                ..FragmentState::default()
            }),
            primitive: PrimitiveState {
                cull_mode: None,
                ..PrimitiveState::default()
            },
            ..RenderPipelineDescriptor::default()
        }
    };
    let paint = cache.queue_render_pipeline(pipeline(
        "drs-paint-engine paint",
        "paint",
        BlendOperation::Max,
    ));
    let erase = cache.queue_render_pipeline(pipeline(
        "drs-paint-engine erase",
        "erase",
        BlendOperation::Min,
    ));
    commands.insert_resource(StrokePipelines {
        layout,
        paint,
        erase,
    });
}

/// The work taken from the main world and not drawn yet, and what drawing it this frame needs.
#[derive(Resource, Default)]
struct Pending {
    /// The work waiting to be drawn.
    waiting: Waiting,
    /// The tiles' uniforms.
    tiles: DynamicUniformBuffer<TileData>,
    /// The segments on the GPU.
    segment_buffer: StorageBuffer<Vec<SegmentData>>,
    /// What this frame draws with, once everything it needs is ready.
    prepared: Option<Prepared>,
    /// Whether the stroke pipeline failed to compile, and that was said.
    failed: bool,
}

/// The work taken from the main world and not drawn yet: the tiles to draw into, in order, and
/// the segments their strokes are made of.
#[derive(Debug, Default)]
struct Waiting {
    /// The segments of every stroke the passes draw.
    segments: Vec<SegmentData>,
    /// The tiles to draw into, in order.
    passes: Vec<Pass>,
}

impl Waiting {
    /// Adds the work handed over in a frame after the work already waiting. A pass that clears
    /// its tile draws everything the passes before it drew there, so it supersedes every earlier
    /// pass for that tile: work waiting for the pipeline holds, for each tile, at most one cleared
    /// pass and those drawn onto it since, and only the segments they draw.
    fn absorb(&mut self, jobs: StrokeJobs) {
        let offset = segment_count(&self.segments);
        self.segments.extend(jobs.segments);
        self.passes.extend(jobs.passes.into_iter().map(|mut pass| {
            for run in &mut pass.runs {
                run.segments = run.segments.start + offset..run.segments.end + offset;
            }
            pass
        }));
        if self.supersede() {
            self.compact();
        }
    }

    /// Drops every pass a later pass that clears the same tile supersedes; whether it dropped any.
    fn supersede(&mut self) -> bool {
        let before = self.passes.len();
        let mut cleared_later = BTreeSet::new();
        let mut kept = Vec::with_capacity(before);
        for pass in std::mem::take(&mut self.passes).into_iter().rev() {
            let image = pass.image.id();
            if cleared_later.contains(&image) {
                continue;
            }
            if pass.clear {
                cleared_later.insert(image);
            }
            kept.push(pass);
        }
        kept.reverse();
        self.passes = kept;
        self.passes.len() != before
    }

    /// Keeps only the segments some pass draws, in their order, and moves the runs onto them.
    fn compact(&mut self) {
        let mut drawn: Vec<Range<u32>> = self
            .passes
            .iter()
            .flat_map(|pass| pass.runs.iter().map(|run| run.segments.clone()))
            .collect();
        drawn.sort_by_key(|range| range.start);
        let mut spans: Vec<Range<u32>> = Vec::new();
        for range in drawn {
            match spans.last_mut() {
                Some(last) if range.start <= last.end => last.end = last.end.max(range.end),
                Some(_) | None => spans.push(range),
            }
        }
        let mut segments = Vec::new();
        let mut moved = Vec::with_capacity(spans.len());
        for span in spans {
            let start = segment_count(&segments);
            if let Some(kept) = self.segments.get(span.start as usize..span.end as usize) {
                segments.extend_from_slice(kept);
            }
            moved.push((span, start));
        }
        for run in self.passes.iter_mut().flat_map(|pass| pass.runs.iter_mut()) {
            let index = moved.partition_point(|(span, _)| span.end <= run.segments.start);
            if let Some((span, start)) = moved.get(index) {
                run.segments =
                    run.segments.start - span.start + start..run.segments.end - span.start + start;
            }
        }
        self.segments = segments;
    }
}

/// Whether the stroke pipeline can draw.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Readiness {
    /// Both variants are compiled.
    Ready,
    /// A variant is still compiling, or waiting for its Shader: the work waits.
    Compiling,
    /// A variant failed to compile, for this reason, and never will: the work is let go.
    Failed(String),
}

impl Readiness {
    /// How ready the pipeline is whose variants are in `states`.
    fn of(states: [&CachedPipelineState; 2]) -> Self {
        let mut readiness = Self::Ready;
        for state in states {
            match state {
                CachedPipelineState::Ok(_) => {}
                CachedPipelineState::Queued
                | CachedPipelineState::Creating(_)
                | CachedPipelineState::Err(
                    ShaderCacheError::ShaderNotLoaded(_)
                    | ShaderCacheError::ShaderImportNotYetAvailable,
                ) => readiness = Self::Compiling,
                CachedPipelineState::Err(error) => return Self::Failed(error.to_string()),
            }
        }
        readiness
    }
}

/// The bind group and each pass's offset into the tiles' uniforms, ready for this frame.
struct Prepared {
    /// The bind group.
    bind_group: BindGroup,
    /// The offset of each pass's tile.
    offsets: Vec<u32>,
}

/// Takes the work handed over in the main world this frame and adds it to the work not drawn
/// yet, so work that could not be drawn in an earlier frame is drawn first.
fn extract_jobs(mut main_world: ResMut<MainWorld>, mut pending: ResMut<Pending>) {
    let Some(mut jobs) = main_world.get_resource_mut::<StrokeJobs>() else {
        return;
    };
    if jobs.passes.is_empty() {
        return;
    }
    let jobs = std::mem::take(&mut *jobs);
    pending.waiting.absorb(jobs);
}

/// Writes the work's buffers and bind group once the pipelines are compiled and every tile's
/// image exists on the GPU; until then the work waits, whole, for a later frame. Should the
/// pipeline fail to compile, the work is let go, so it holds no tile's image, and the failure
/// is said once.
fn prepare_jobs(
    mut pending: ResMut<Pending>,
    pipelines: Option<Res<StrokePipelines>>,
    cache: Res<PipelineCache>,
    images: Res<RenderAssets<GpuImage>>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    pending.prepared = None;
    if pending.waiting.passes.is_empty() {
        return;
    }
    let Some(pipelines) = pipelines else {
        return;
    };
    match Readiness::of([
        cache.get_render_pipeline_state(pipelines.paint),
        cache.get_render_pipeline_state(pipelines.erase),
    ]) {
        Readiness::Ready => {}
        Readiness::Compiling => return,
        Readiness::Failed(reason) => {
            if !pending.failed {
                log::warn!(
                    "the stroke pipeline failed to compile, so Terrain is not drawn: {reason}"
                );
                pending.failed = true;
            }
            pending.waiting = Waiting::default();
            return;
        }
    }
    if !pending
        .waiting
        .passes
        .iter()
        .all(|pass| images.get(&pass.image).is_some())
    {
        return;
    }
    let pending = &mut *pending;
    pending.tiles.clear();
    let offsets: Vec<u32> = pending
        .waiting
        .passes
        .iter()
        .map(|pass| {
            pending.tiles.push(&TileData {
                origin: IVec2::new(pass.key.x, pass.key.y) * tile_side(),
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "a band is a small whole number, exact in f32"
                )]
                band: pass.band as f32,
                padding: 0.0,
            })
        })
        .collect();
    pending.tiles.write_buffer(&device, &queue);
    let mut segments = pending.waiting.segments.clone();
    if segments.is_empty() {
        // A storage binding may not be empty; no run draws this one.
        segments.push(SegmentData {
            start: Vec2::ZERO,
            end: Vec2::ZERO,
            radius: 0.0,
            hardness: 0.0,
            strength: 0.0,
            padding: 0.0,
        });
    }
    pending.segment_buffer.set(segments);
    pending.segment_buffer.write_buffer(&device, &queue);
    let (Some(tiles), Some(segments)) = (pending.tiles.binding(), pending.segment_buffer.buffer())
    else {
        return;
    };
    let bind_group = device.create_bind_group(
        "drs-paint-engine strokes",
        &cache.get_bind_group_layout(&pipelines.layout),
        &BindGroupEntries::sequential((tiles, segments.as_entire_binding())),
    );
    pending.prepared = Some(Prepared {
        bind_group,
        offsets,
    });
}

/// How many pixels a side a tile has, as a whole-pixel index.
fn tile_side() -> i32 {
    i32::try_from(COVERAGE_TILE_PIXELS).unwrap_or(i32::MAX)
}

/// Draws the prepared work into the tiles' images, one pass per tile, and lets it go.
fn rasterize_jobs(
    mut context: RenderContext,
    mut pending: ResMut<Pending>,
    pipelines: Option<Res<StrokePipelines>>,
    cache: Res<PipelineCache>,
    images: Res<RenderAssets<GpuImage>>,
) {
    let Some(prepared) = pending.prepared.take() else {
        return;
    };
    let Some(pipelines) = pipelines else {
        return;
    };
    let (Some(paint), Some(erase)) = (
        cache.get_render_pipeline(pipelines.paint),
        cache.get_render_pipeline(pipelines.erase),
    ) else {
        return;
    };
    let passes = std::mem::take(&mut pending.waiting).passes;
    let encoder = context.command_encoder();
    for (pass, offset) in passes.iter().zip(&prepared.offsets) {
        let Some(target) = images.get(&pass.image) else {
            continue;
        };
        #[expect(
            clippy::default_trait_access,
            reason = "the clear colour is wgpu's, which the render stack does not name; its \
                      default is nothing"
        )]
        let load = if pass.clear {
            LoadOp::Clear(Default::default())
        } else {
            LoadOp::Load
        };
        let mut drawing = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("drs-paint-engine tile"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &target.texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        drawing.set_bind_group(0, Some(&*prepared.bind_group), &[*offset]);
        let mut erasing = None;
        for run in &pass.runs {
            if erasing != Some(run.erase) {
                drawing.set_pipeline(if run.erase { erase } else { paint });
                erasing = Some(run.erase);
            }
            drawing.draw(0..6, run.segments.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::missing_panics_doc,
        reason = "a test stops at the first thing that is not as expected"
    )]
    use super::*;
    use bevy_shader::ShaderCacheError;
    use drs_model::BrushSettings;

    /// Three strokes, the last erasing, of one, two, and three segments.
    fn strokes() -> Vec<Stroke> {
        let brush = BrushSettings {
            size: 1.0,
            hardness: 0.5,
            strength: 1.0,
        };
        let stroke = |points: &[Vec2], erase| Stroke {
            points: points.to_vec(),
            brush,
            erase,
        };
        vec![
            stroke(&[Vec2::new(1.0, 1.0)], false),
            stroke(&[Vec2::ZERO, Vec2::X, Vec2::ONE], false),
            stroke(&[Vec2::ZERO, Vec2::Y, Vec2::ONE, Vec2::X], true),
        ]
    }

    /// The work of one frame: each of `drawn` names a tile's image, whether it is cleared, and
    /// the strokes drawn into it.
    fn frame(strokes: &[Stroke], drawn: &[(&Handle<Image>, bool, &[usize])]) -> StrokeJobs {
        let mut jobs = StrokeJobs::default();
        jobs.hand_over(
            strokes,
            drawn
                .iter()
                .map(|(image, clear, numbers)| Drawn {
                    image: (*image).clone(),
                    key: TileKey { x: 0, y: 0 },
                    band: 32,
                    clear: *clear,
                    strokes: numbers.to_vec(),
                })
                .collect(),
        );
        jobs
    }

    /// A run as drawn: whether it erases, and its segments.
    type RunDrawn = (bool, Vec<SegmentData>);

    /// What each waiting pass draws: its image, whether it clears, and the segments of its
    /// runs, with whether each erases.
    fn drawn(waiting: &Waiting) -> Vec<(AssetId<Image>, bool, Vec<RunDrawn>)> {
        waiting
            .passes
            .iter()
            .map(|pass| {
                let runs = pass
                    .runs
                    .iter()
                    .map(|run| {
                        let range = run.segments.start as usize..run.segments.end as usize;
                        (run.erase, waiting.segments[range].to_vec())
                    })
                    .collect();
                (pass.image.id(), pass.clear, runs)
            })
            .collect()
    }

    /// The segments of the strokes numbered in `numbers`, in order.
    fn of(strokes: &[Stroke], numbers: &[usize]) -> Vec<SegmentData> {
        numbers
            .iter()
            .flat_map(|number| segments(&strokes[*number]))
            .collect()
    }

    /// Work handed over in a later frame waits after the work of earlier frames, each pass still
    /// drawing its own strokes' segments.
    #[test]
    fn work_waits_in_the_order_it_came() {
        let strokes = strokes();
        let mut images = Assets::<Image>::default();
        let (first, second) = (images.add(Image::default()), images.add(Image::default()));
        let mut waiting = Waiting::default();

        waiting.absorb(frame(&strokes, &[(&first, true, &[0, 1])]));
        waiting.absorb(frame(&strokes, &[(&second, false, &[1, 2])]));

        assert_eq!(
            drawn(&waiting),
            vec![
                (first.id(), true, vec![(false, of(&strokes, &[0, 1]))]),
                (
                    second.id(),
                    false,
                    vec![(false, of(&strokes, &[1])), (true, of(&strokes, &[2]))]
                ),
            ]
        );
    }

    /// A pass that clears a tile lets go of every pass for that tile still waiting and of the
    /// segments only they drew, and keeps what other tiles wait for and what is drawn onto it
    /// later.
    #[test]
    fn a_cleared_tile_supersedes_its_waiting_work() {
        let strokes = strokes();
        let mut images = Assets::<Image>::default();
        let (first, second) = (images.add(Image::default()), images.add(Image::default()));
        let mut waiting = Waiting::default();

        waiting.absorb(frame(
            &strokes,
            &[(&first, true, &[0, 2]), (&second, true, &[1])],
        ));
        waiting.absorb(frame(&strokes, &[(&first, false, &[1])]));
        waiting.absorb(frame(&strokes, &[(&first, true, &[0])]));
        waiting.absorb(frame(&strokes, &[(&first, false, &[2])]));

        assert_eq!(
            drawn(&waiting),
            vec![
                (second.id(), true, vec![(false, of(&strokes, &[1]))]),
                (first.id(), true, vec![(false, of(&strokes, &[0]))]),
                (first.id(), false, vec![(true, of(&strokes, &[2]))]),
            ]
        );
        assert_eq!(
            waiting.segments.len(),
            of(&strokes, &[1, 0, 2]).len(),
            "only the segments still drawn are kept"
        );
    }

    /// Work waits while either variant of the pipeline is queued, compiling, or waiting for its
    /// Shader, and is let go once either has failed for good.
    #[test]
    fn work_waits_for_the_pipeline_unless_it_failed() {
        let queued = CachedPipelineState::Queued;
        let no_shader = CachedPipelineState::Err(ShaderCacheError::ShaderNotLoaded(SHADER.id()));
        let broken = CachedPipelineState::Err(ShaderCacheError::CreateShaderModule(
            "no entry point".to_owned(),
        ));
        assert_eq!(Readiness::of([&queued, &queued]), Readiness::Compiling);
        assert_eq!(Readiness::of([&no_shader, &queued]), Readiness::Compiling);
        assert!(matches!(
            Readiness::of([&queued, &broken]),
            Readiness::Failed(reason) if reason.contains("no entry point")
        ));
    }
}
