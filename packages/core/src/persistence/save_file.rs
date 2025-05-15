use crate::components::{Layer as LayerComponent, Level as LevelComponent, Project, Texture};
use crate::constants;
use crate::persistence::entities::{image::Image, layer::Layer, level::Level};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveFile {
    pub version: String,
    pub name: String,
    pub size: Rect,
    pub levels: Vec<Level>,
}

impl SaveFile {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        project_query: Query<(&Project, &Name, &Children), With<Project>>,
        children_query: Query<&Children>,
        level_query: Query<&Name, With<LevelComponent>>,
        layer_query: Query<(&LayerComponent, &Name)>,
        mesh_query: Query<(&Texture, &Name), With<Mesh2d>>,
        transform_query: Query<&Transform>,
        material_query: Query<&MeshMaterial2d<ColorMaterial>>,
        materials: &Res<Assets<ColorMaterial>>,
    ) -> Result<Self, BevyError> {
        let (project, project_name, project_children) = project_query.single()?;
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
                    let (texture, name) = mesh_query.get(entity)?;
                    let transform = transform_query.get(entity)?;
                    let material_handle = material_query.get(entity)?;

                    if let Some(material) = materials.get(material_handle) {
                        if let Some(texture_handle) = &material.texture {
                            if let Some(path) = texture_handle.path() {
                                images.push(Image {
                                    name: name.to_string(),
                                    path: path.path().to_path_buf(),
                                    alpha: material.color.alpha(),
                                    size: texture.size,
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
            version: String::from(constants::VERSION),
            name: project_name.to_string(),
            size: project.size,
            levels,
        })
    }

    pub fn restore(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        asset_server: &Res<AssetServer>,
    ) {
        let mut project = commands.spawn((
            Name::new(self.name.clone()),
            Project::new(self.name.clone(), self.size),
        ));

        project.with_children(|project| {
            for level in &self.levels {
                let mut parent = project.spawn((Name::new(level.name.clone()), LevelComponent));

                parent.with_children(|parent| {
                    for layer in &level.layers {
                        let mut child = parent.spawn((
                            Name::new(layer.name.clone()),
                            LayerComponent {
                                weight: layer.weight,
                            },
                        ));

                        child.with_children(|grand_child| {
                            for image in &layer.images {
                                grand_child.spawn((
                                    Name::new(image.name.clone()),
                                    Mesh2d(meshes.add(image.size)),
                                    MeshMaterial2d(materials.add(ColorMaterial {
                                        texture: Some(asset_server.load(image.path.clone())),
                                        ..default()
                                    })),
                                    image.transform,
                                ));
                            }
                        });
                    }
                });
            }
        });
    }
}
