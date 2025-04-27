use bevy::prelude::Resource;

#[derive(Resource)]
pub struct CameraControlsState {
    pub(crate) moving: bool,
    pub(crate) locked: bool,
}
