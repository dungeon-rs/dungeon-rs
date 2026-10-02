//! Export Level: the Bounds of a Level rendered tile by tile and streamed into a PNG.
//!
//! An Export takes frames: the render Engine draws a tile offscreen and hands its pixels back a
//! few frames later. So an Export is a job that [`handle_export_level`] starts on request and
//! advances every frame: it asks the Engine for the next tile as soon as the Engine takes a
//! request, writes each tile as its pixels arrive, in row-major order from the top-left corner
//! of the image, and finishes the image once the last tile is written. A failure anywhere drops
//! the image writer, which removes the partial file, and is answered with its reason. Nothing
//! is recorded in the history: an Export changes nothing in the Project.

use crate::ProjectManagerError;
use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::message::MessageReader;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::SystemState;
use bevy_ecs::world::World;
use bevy_math::Vec2;
use drs_model::{
    Bounds, ExportLevel, ExportRefused, Level, LevelExported, with_extension_if_missing,
};
use drs_output_access::{ImageWriter, OutputError, Tile, begin_image, finish_image, write_tile};
use drs_render_engine::{
    MOST_TILE_PIXELS, RegionRequest, RenderError, release_regions, request_region, take_region,
};
use std::collections::VecDeque;
use std::path::PathBuf;

/// Why an Export could not be made; answered through [`ProjectManagerError::Export`].
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    /// The resolution is below the least or above the most the Export supports.
    #[error(
        "a resolution of {pixels_per_cell} pixels per cell is outside the limits: the Export \
         supports {least} to {most} pixels per cell"
    )]
    ResolutionOutsideLimits {
        /// The resolution asked for.
        pixels_per_cell: u32,
        /// The lowest resolution supported.
        least: u32,
        /// The highest resolution supported.
        most: u32,
    },
    /// The entity to export is not a Level.
    #[error("the entity to export is not a Level")]
    NotALevel,
    /// The Level belongs to no Project, so there are no Bounds to export.
    #[error("the Level belongs to no Project")]
    NoProject,
    /// The Bounds at the resolution make an image whose pixel size cannot be counted.
    #[error(
        "the Bounds of {width} by {height} cells at {pixels_per_cell} pixels per cell are too large to export"
    )]
    TooLarge {
        /// The Bounds' width in cells.
        width: u32,
        /// The Bounds' height in cells.
        height: u32,
        /// The resolution asked for.
        pixels_per_cell: u32,
    },
    /// The image could not be written.
    #[error(transparent)]
    Output(#[from] OutputError),
    /// A tile could not be rendered.
    #[error(transparent)]
    Render(#[from] RenderError),
}

/// The Exports in progress, worked on one at a time in the order they were asked for.
#[derive(Resource, Default)]
pub(crate) struct Exports {
    /// The Export being made is at the front.
    queue: VecDeque<Box<Export>>,
}

/// An Export in progress.
struct Export {
    /// The Level being exported.
    level: Entity,
    /// The file as the request named it, echoed when the Export fails.
    asked: PathBuf,
    /// The file being written, `.png` included.
    path: PathBuf,
    /// The image's width in pixels.
    width: u32,
    /// The image's height in pixels.
    height: u32,
    /// How many image pixels one cell spans.
    pixels_per_cell: u32,
    /// The side of a tile in pixels.
    tile_size: u32,
    /// The Bounds being exported.
    bounds: Bounds,
    /// How many tiles make one row of the image.
    tiles_across: u32,
    /// How many tiles the image takes.
    tiles: u32,
    /// How many tiles have been asked of the Engine, in row-major order.
    requested: u32,
    /// How many tiles have been written, in the same order.
    written: u32,
    /// The requests whose pixels have not arrived yet, oldest first.
    pending: VecDeque<RegionRequest>,
    /// The image being written.
    writer: ImageWriter,
}

/// What a frame's work on an Export came to.
enum Step {
    /// The Export goes on next frame.
    Working(Box<Export>),
    /// The image is written.
    Finished(LevelExported),
}

/// An Export that failed, with what the answer needs.
struct Failure {
    /// The Level that was to be exported.
    level: Entity,
    /// The file as the request named it.
    path: PathBuf,
    /// Why.
    error: ProjectManagerError,
}

/// Starts an Export for every [`ExportLevel`] request and advances the Export in progress by
/// a frame, answering each with [`LevelExported`] or [`ExportRefused`].
pub(crate) fn handle_export_level(
    world: &mut World,
    requests: &mut SystemState<MessageReader<ExportLevel>>,
) {
    let requests: Vec<ExportLevel> = match requests.get_mut(world) {
        Ok(mut reader) => reader.read().cloned().collect(),
        Err(_) => return,
    };
    for request in requests {
        match begin(world, &request) {
            Ok(export) => {
                world
                    .get_resource_or_init::<Exports>()
                    .queue
                    .push_back(Box::new(export));
            }
            Err(error) => {
                world.write_message(ExportRefused {
                    level: request.level,
                    path: request.path,
                    reason: error.to_string(),
                });
            }
        }
    }
    advance(world);
}

/// Checks a request and opens its image: the resolution within its limits, the tile size one
/// the Engine renders, the Level with its Project's Bounds, and the image as many pixels as the
/// Bounds are cells times the resolution.
///
/// # Errors
///
/// The refusal, before any file is created, or the output's error when the image cannot be
/// opened.
fn begin(world: &mut World, request: &ExportLevel) -> Result<Export, ProjectManagerError> {
    let pixels_per_cell = request.pixels_per_cell;
    if !(ExportLevel::LEAST_PIXELS_PER_CELL..=ExportLevel::MOST_PIXELS_PER_CELL)
        .contains(&pixels_per_cell)
    {
        return Err(ExportError::ResolutionOutsideLimits {
            pixels_per_cell,
            least: ExportLevel::LEAST_PIXELS_PER_CELL,
            most: ExportLevel::MOST_PIXELS_PER_CELL,
        }
        .into());
    }
    let tile_size = request.tile_size;
    if tile_size == 0 || tile_size > MOST_TILE_PIXELS {
        return Err(ExportError::from(RenderError::BadTileSize(tile_size)).into());
    }
    if world.get::<Level>(request.level).is_none() {
        return Err(ExportError::NotALevel.into());
    }
    let bounds = world
        .get::<ChildOf>(request.level)
        .map(ChildOf::parent)
        .and_then(|project| world.get::<Bounds>(project))
        .copied()
        .ok_or(ExportError::NoProject)?;
    let too_large = || ExportError::TooLarge {
        width: bounds.size.x,
        height: bounds.size.y,
        pixels_per_cell,
    };
    let width = bounds
        .size
        .x
        .checked_mul(pixels_per_cell)
        .ok_or_else(too_large)?;
    let height = bounds
        .size
        .y
        .checked_mul(pixels_per_cell)
        .ok_or_else(too_large)?;
    let tiles_across = width.div_ceil(tile_size);
    let tiles = tiles_across
        .checked_mul(height.div_ceil(tile_size))
        .ok_or_else(too_large)?;
    let path = with_extension_if_missing(request.path.clone(), "png");
    let writer = begin_image(&path, width, height).map_err(ExportError::from)?;
    Ok(Export {
        level: request.level,
        asked: request.path.clone(),
        path,
        width,
        height,
        pixels_per_cell,
        tile_size,
        bounds,
        tiles_across,
        tiles,
        requested: 0,
        written: 0,
        pending: VecDeque::new(),
        writer,
    })
}

/// Advances the Export at the front of the queue by a frame and answers it when it ends.
fn advance(world: &mut World) {
    let Some(export) = world.get_resource_or_init::<Exports>().queue.pop_front() else {
        return;
    };
    match export.step(world) {
        Ok(Step::Working(export)) => {
            world
                .get_resource_or_init::<Exports>()
                .queue
                .push_front(export);
        }
        Ok(Step::Finished(exported)) => {
            release_regions(world);
            world.write_message(exported);
        }
        Err(Failure { level, path, error }) => {
            release_regions(world);
            world.write_message(ExportRefused {
                level,
                path,
                reason: error.to_string(),
            });
        }
    }
}

impl Export {
    /// A frame's work: writes every tile whose pixels arrived, asks for the next tile when the
    /// Engine can take it, and finishes the image once every tile is written.
    ///
    /// On failure the writer goes with the Export, which removes the partial file.
    ///
    /// # Errors
    ///
    /// Why the Export failed, with the Level and path the answer names.
    fn step(mut self: Box<Self>, world: &mut World) -> Result<Step, Failure> {
        match self.work(world) {
            Ok(true) => {
                let Self {
                    level,
                    asked,
                    path,
                    width,
                    height,
                    writer,
                    ..
                } = *self;
                match finish_image(writer) {
                    Ok(()) => Ok(Step::Finished(LevelExported {
                        level,
                        path,
                        width,
                        height,
                    })),
                    Err(error) => Err(Failure {
                        level,
                        path: asked,
                        error: ExportError::from(error).into(),
                    }),
                }
            }
            Ok(false) => Ok(Step::Working(self)),
            Err(error) => Err(Failure {
                level: self.level,
                path: self.asked,
                error: error.into(),
            }),
        }
    }

    /// Writes the tiles that arrived and requests the next one; `true` once every tile is
    /// written.
    ///
    /// # Errors
    ///
    /// The Engine's error when a tile cannot be rendered or read back, or the output's when it
    /// cannot be written.
    fn work(&mut self, world: &mut World) -> Result<bool, ExportError> {
        while let Some(&request) = self.pending.front() {
            let Some(pixels) = take_region(world, request)? else {
                break;
            };
            self.pending.pop_front();
            let (x, y) = self.pixel_corner(self.written);
            write_tile(
                &mut self.writer,
                Tile {
                    x,
                    y,
                    width: pixels.size,
                    height: pixels.size,
                    rgba: &pixels.rgba,
                },
            )?;
            self.written += 1;
        }
        if self.written == self.tiles {
            return Ok(true);
        }
        if self.requested < self.tiles {
            let bottom_left = self.bottom_left(self.requested);
            match request_region(world, bottom_left, self.pixels_per_cell, self.tile_size) {
                Ok(request) => {
                    self.pending.push_back(request);
                    self.requested += 1;
                }
                Err(RenderError::RegionPending) => {}
                Err(error) => return Err(error.into()),
            }
        }
        Ok(false)
    }

    /// The top-left pixel of a tile, counted row-major from the top-left corner of the image.
    fn pixel_corner(&self, tile: u32) -> (u32, u32) {
        let column = tile % self.tiles_across;
        let row = tile / self.tiles_across;
        (column * self.tile_size, row * self.tile_size)
    }

    /// The lower-left corner, in cells, of the square of the Level a tile shows: the tile's
    /// column from the Bounds' left edge, and its row down from the Bounds' top edge.
    ///
    /// The corner is counted in whole pixels from the Bounds' lower-left corner and divided by
    /// the resolution once per axis, so every tile's edge lands where the image's pixel grid
    /// says, whether or not the resolution divides the tile size; the bottom row of tiles may
    /// reach below the Bounds.
    fn bottom_left(&self, tile: u32) -> Vec2 {
        let (x, y) = self.pixel_corner(tile);
        let left = i64::from(x);
        let bottom = i64::from(self.height) - i64::from(y) - i64::from(self.tile_size);
        #[expect(
            clippy::cast_precision_loss,
            reason = "an image is far fewer pixels a side than f32 counts exactly"
        )]
        let cells = |pixels: i64| pixels as f32 / self.pixels_per_cell as f32;
        self.bounds.origin.as_vec2() + Vec2::new(cells(left), cells(bottom))
    }
}
