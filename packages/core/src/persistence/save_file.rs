use crate::components::{Layer as LayerComponent, Level as LevelComponent, Project};
use crate::persistence::entities::{image::Image, layer::Layer, level::Level};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveFile {
    pub version: &'static str,
    pub size: Rect,
    pub levels: Vec<Level>,
}

impl SaveFile {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        project_query: Query<&Children, With<Project>>,
        children_query: Query<&Children>,
        level_query: Query<&Name, With<LevelComponent>>,
        layer_query: Query<(&LayerComponent, &Name)>,
        mesh_query: Query<&Mesh2d>,
        transform_query: Query<&Transform>,
        material_query: Query<&MeshMaterial2d<ColorMaterial>>,
        materials: &Res<Assets<ColorMaterial>>,
    ) -> Result<Self, BevyError> {
        let project_children = project_query.single()?;
        let mut levels = Vec::new();

        for level_entity in project_children.iter() {
            let level_name = level_query.get(level_entity)?;
            let mut layers = Vec::new();

            let level_children = children_query.get(level_entity)?;
            for layer_entity in level_children.iter() {
                let (layer_component, layer_name) = layer_query.get(layer_entity)?;

                let mut images = Vec::new();
                let layer_children = children_query.get(layer_entity)?;
                for entity in layer_children.iter() {
                    let _mesh = mesh_query.get(entity)?;
                    let transform = transform_query.get(entity)?;
                    let material_handle = material_query.get(entity)?;

                    if let Some(material) = materials.get(material_handle) {
                        if let Some(texture_handle) = &material.texture {
                            if let Some(path) = texture_handle.path() {
                                images.push(Image {
                                    path: path.path().to_path_buf(),
                                    alpha: material.color.alpha(),
                                    transform: *transform,
                                });
                            }
                        }
                    }
                }

                layers.push(Layer::new(
                    layer_name.as_str(),
                    layer_component.weight,
                    images,
                ));
            }

            levels.push(Level::new(level_name.as_str(), layers));
        }

        Ok(Self {
            version: "0.0.1",
            size: Rect::from_center_size(Vec2::ZERO, Vec2::splat(100.0)),
            levels,
        })
    }
}
