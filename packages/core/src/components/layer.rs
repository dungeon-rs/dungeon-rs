use bevy::prelude::{Component, Transform, Visibility};

/// Not to be confused with Bevy's Layer component.
///
/// The [Layer] component marks a logical grouping of entities that can be toggled
/// as one entity.
///
/// Because this technically has a [Transform], we could allow the user to "move" entire layers
/// around at once, though I'm not sure if that's an intended use case.
#[derive(Component, Default)]
#[component(immutable)]
#[require(Transform::from_xyz(0.0, 0.0, 0.0), Visibility::default())]
pub struct Layer {
    /// The weight determines the order in which layers are rendered.
    /// A higher weight will render on top, making this an abstraction of the Z index.
    pub weight: i32,
}
