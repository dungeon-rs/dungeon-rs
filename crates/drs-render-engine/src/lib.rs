#![doc = include_str!("../README.md")]

mod projection;
mod props;

use bevy_app::{App, Plugin, PostUpdate, Startup};
use bevy_camera::CameraUpdateSystems;
use bevy_ecs::schedule::common_conditions::{any_with_component, resource_exists_and_changed};
use bevy_ecs::schedule::{IntoScheduleConfigs, SystemCondition};
use bevy_transform::TransformSystems;
use drs_model::Viewport;

/// Draws the Level: the viewport systems that keep one sprite per Prop, and the projection
/// that follows the [`Viewport`].
pub struct RenderEnginePlugin;

impl Plugin for RenderEnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, projection::spawn_camera)
            .add_systems(
                PostUpdate,
                (
                    projection::follow_viewport.run_if(
                        resource_exists_and_changed::<Viewport>
                            .or_eager(projection::target_resized),
                    ),
                    props::sync_props.run_if(props::props_changed),
                    props::settle_loads.run_if(any_with_component::<props::Loading>),
                )
                    .chain()
                    .before(TransformSystems::Propagate)
                    .before(CameraUpdateSystems),
            );
    }
}
