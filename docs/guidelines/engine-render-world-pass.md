# Engine render-world pass

**Use when**: an Engine draws into images on the GPU in the render world, from work its verb plans in the main world (PaintEngine rasterizing Terrain tiles). **Not when**: the Engine draws the model through cameras and Materials (RenderEngine's viewport systems and RenderRegion), or the work needs no GPU (a CPU verb such as Rasterize).
**Exemplar**: `crates/drs-paint-engine/src/gpu.rs`

## Rules

- The verb hands its work over through a `SystemParam` the Engine defines (`StrokeRasterizer`), which the Manager's system takes and passes on, holding a `pub(crate)` main-world resource of the work (`StrokeJobs`) as an `Option`; never an event, a message, or an observer. The plugin inits that resource only when a `RenderApp` exists, so the parameter tells the verb whether there is a GPU, and the Engine's caches settle that once, when made, as the [Engine cache held by a Manager](./engine-cache-held-by-a-manager.md) guideline says.
- The plugin's `build` returns early without a `RenderApp`, and otherwise queues its pipelines in `RenderStartup`, extracts in `ExtractSchedule` by taking the main world's work whole (`std::mem::take` through `MainWorld`) into a render-world resource of work not yet drawn (`Pending`), prepares buffers and bind groups in `RenderSystems::PrepareBindGroups`, and encodes its passes through `RenderContext` in `RenderGraphSystems::Begin`, which runs before any camera renders, so every image is complete before anything samples it and only `bevy_render` is needed.
- Work waits, whole, while a pipeline is queued, compiling, or waiting for its Shader, or while an image it draws into is not yet on the GPU; a pipeline that failed for good (an error other than a Shader not loaded yet) lets the work go and warns once. A pass that redraws an image from nothing supersedes every pass for that image still waiting, so waiting work, and the image handles it holds, stay bounded.
- The images drawn into are made for the render world alone (`RenderAssetUsages::RENDER_WORLD`, usable as a render attachment, a texture binding, and a copy source so a test can read one back), held by strong handle in the Engine's cache so dropping the cache frees them, and named in the model by the bits of their asset index (`GpuTile`), which another Engine maps back to the image.
- The Shader follows the [built-in Material](./built-in-material.md) guideline's Shader rules, a fixed handle added in `finish`; without it the plugin removes its main-world work resource, so the verb falls back to the CPU rather than queuing work nothing can draw.
- Unit tests check the planning and the waiting as plain functions (handing over, superseding, readiness from pipeline states); the seams read the images back from the GPU and compare them with the CPU reference, and fail rather than skip without an adapter.

## Example

```rust
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
}

/// The work rasterizing on the GPU has been handed in the frame and not yet taken to the render
/// world: the tiles to draw into, in order, and the segments their strokes are made of. The
/// render world takes it once a frame; a Terrain's cache adds to it through
/// [`StrokeRasterizer`].
#[derive(Resource, Debug, Default)]
pub(crate) struct StrokeJobs {
    /// The segments of every stroke the passes draw.
    segments: Vec<SegmentData>,
    /// The tiles to draw into, in the order they are drawn.
    passes: Vec<Pass>,
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
```

## Pitfalls

- Ordering the pass with a render-graph node or a `bevy_core_pipeline` set: Begin, in `bevy_render`, already runs before every camera.
- Dropping work because its pipeline is not compiled yet: the images then stay blank until something redraws them; only a pipeline that failed for good lets work go.
- Keeping every frame's work while waiting: a drag hands over its tiles each frame, and without superseding the waiting work and the images it holds grow without bound.
- Holding an image by weak handle in waiting work: an image not yet uploaded and one already freed look alike, so the work waits forever.
