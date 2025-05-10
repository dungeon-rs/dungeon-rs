use crate::export::{ExportRequest, ExportStatus};
use bevy::asset::RenderAssetUsages;
use bevy::image::BevyDefault;
use bevy::prelude::{Assets, Handle, Image, ResMut, Resource, Vec2, default};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use std::collections::VecDeque;
use std::path::PathBuf;

#[derive(Debug, Resource)]
pub(crate) struct Screenshot {
    /// The size of each frame the screenshot consists of.
    pub frame_size: (u16, u16),
    /// The path to the final export image.
    pub output: PathBuf,
    /// A list of coordinates where a screenshot needs to be taken.
    pub coordinates: VecDeque<Vec2>,
    /// The internal buffer of already completed images, contains raw pixel data in each entry.
    pub buffer: Vec<Vec<u8>>,
    /// The image handle where Bevy renders the camera output into.
    pub render_target: Handle<Image>,
    /// The state of the screenshot, see [ExportStatus] for the different states.
    pub state: ExportStatus,
}

impl Screenshot {
    pub fn new(request: &ExportRequest, mut images: ResMut<Assets<Image>>) -> Self {
        // TODO: for now we'll assume the canvas is *always* 2048 pixels.
        let canvas: (u16, u16) = (2048, 2048);

        let frames_x = (canvas.0 as f32 / request.frame_size.0 as f32).ceil() as u16;
        let frames_y = (canvas.1 as f32 / request.frame_size.1 as f32).ceil() as u16;

        let mut coordinates = VecDeque::new();
        for y in 0..frames_y {
            for x in 0..frames_x {
                let pos_x = x as f32 * request.frame_size.0 as f32;
                let pos_y = y as f32 * request.frame_size.1 as f32;
                coordinates.push_back(Vec2::new(pos_x, pos_y));
            }
        }

        let mut render_image = Image::new_fill(
            Extent3d {
                width: request.frame_size.0 as u32,
                height: request.frame_size.1 as u32,
                ..default()
            },
            TextureDimension::D2,
            &[0; 4],
            TextureFormat::bevy_default(),
            RenderAssetUsages::default(),
        );
        render_image.texture_descriptor.usage |= TextureUsages::COPY_SRC
            | TextureUsages::RENDER_ATTACHMENT
            | TextureUsages::TEXTURE_BINDING;

        let frame_count = coordinates.len();
        Self {
            frame_size: request.frame_size,
            output: request.output.clone(),
            coordinates,
            buffer: Vec::with_capacity(frame_count),
            render_target: images.add(render_image),
            state: ExportStatus::Preparing,
        }
    }
}
