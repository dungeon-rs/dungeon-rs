//! The thumbnails of an Asset Folder's Assets, as the browser shows them.

use bevy_ecs::component::Component;
use bevy_ecs::reflect::ReflectComponent;
use bevy_math::UVec2;
use bevy_reflect::Reflect;

/// The name of the asset source thumbnails are read through: paths read
/// `thumb://<folder-key>/<place>`, naming the Asset whose thumbnail is wanted.
pub const THUMBNAIL_SOURCE: &str = "thumb";

/// Where an Asset's thumbnail stands.
#[derive(Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ThumbnailState {
    /// Not generated yet.
    #[default]
    Pending,
    /// Generated, and this many pixels wide and high.
    Ready(UVec2),
    /// The Asset's file could not be decoded as an image.
    Broken,
}

/// The thumbnail state of each Asset of the Asset Folder on the same entity, in the order of
/// its index of Assets. Written by the library Manager.
#[derive(Component, Reflect, Debug, Clone, Default, PartialEq, Eq)]
#[reflect(Component)]
pub struct Thumbnails {
    /// One state per Asset, in the order of the folder's index.
    pub states: Vec<ThumbnailState>,
}
