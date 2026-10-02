#![doc = include_str!("../README.md")]

mod projection;
mod props;
mod region;

pub use region::{
    MOST_TILE_PIXELS, RegionPixels, RegionRequest, RenderError, release_regions, request_region,
    take_region,
};

use bevy_app::{App, Plugin, PostUpdate, Startup};
use bevy_camera::CameraUpdateSystems;
use bevy_ecs::schedule::common_conditions::{
    any_with_component, resource_exists, resource_exists_and_changed,
};
use bevy_ecs::schedule::{IntoScheduleConfigs, SystemCondition};
use bevy_render::renderer::render_system;
use bevy_render::{ExtractSchedule, Render, RenderApp, RenderSystems};
use bevy_transform::TransformSystems;
use drs_model::Viewport;

/// Draws the Level: the viewport systems that keep one sprite per Prop, the projection that
/// follows the [`Viewport`], and the offscreen rendering of regions for the Export.
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
                    region::gate_regions.run_if(resource_exists::<region::Offscreen>),
                )
                    .chain()
                    .before(TransformSystems::Propagate)
                    .before(CameraUpdateSystems),
            );
        // Without a renderer there is nothing to draw regions with, and `request_region` says so.
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<region::Readbacks>()
                .add_systems(ExtractSchedule, region::extract_regions)
                .add_systems(
                    Render,
                    region::copy_regions
                        .in_set(RenderSystems::Render)
                        .after(render_system),
                );
            app.init_resource::<region::Offscreen>();
        }
    }
}
