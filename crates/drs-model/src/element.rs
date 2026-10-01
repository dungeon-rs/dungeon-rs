//! Elements, their stable identity, and the registry of Element kinds.

use crate::AssetReferenceRow;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::error::BevyError;
use bevy_ecs::reflect::ReflectComponent;
use bevy_ecs::resource::Resource;
use bevy_ecs::world::World;
use bevy_math::Vec2;
use bevy_reflect::Reflect;
use drs_history::{HistoryError, Target};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::BTreeMap;

/// The identity of an Element, stable across saving, loading, undo, and redo.
///
/// Commands and history address Elements by this, never by the entity handle, which changes
/// whenever an Element is respawned.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[reflect(Component, Hash, PartialEq)]
pub struct ElementId(u128);

impl ElementId {
    /// A fresh identity that no other Element has.
    #[must_use]
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().as_u128())
    }

    /// The identity behind a stored number.
    #[must_use]
    pub const fn from_raw(raw: u128) -> Self {
        Self(raw)
    }

    /// The number to store.
    #[must_use]
    pub const fn as_raw(self) -> u128 {
        self.0
    }
}

impl Default for ElementId {
    fn default() -> Self {
        Self::new()
    }
}

impl Target for ElementId {
    fn entity(&self, world: &mut World) -> Result<Entity, BevyError> {
        world
            .query::<(Entity, &ElementId)>()
            .iter(world)
            .find(|(_, id)| *id == self)
            .map(|(entity, _)| entity)
            .ok_or_else(|| HistoryError::MissingTarget.into())
    }
}

/// What every Element has, whatever its kind.
#[derive(Component, Reflect, Debug, Clone, PartialEq)]
#[reflect(Component)]
#[require(ElementId)]
pub struct Element {
    /// Which kind of Element this is; the kind's own component sits beside this one.
    pub kind: ElementKindName,
    /// The centre of the Element in Grid cells, measured from the Level's origin.
    pub position: Vec2,
    /// The width and height of the Element in Grid cells.
    pub size: Vec2,
}

/// A single placed image. Its default Material shows the image.
#[derive(Component, Reflect, Debug, Clone, PartialEq, Eq)]
#[reflect(Component)]
pub struct Prop {
    /// The row of the Project's Asset Reference table that names the image.
    pub asset: AssetReferenceRow,
}

/// The name of an Element kind, such as `prop`; a Plugin's kind is `<plugin>/<kind>`.
#[derive(Reflect, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ElementKindName(Cow<'static, str>);

impl ElementKindName {
    /// A kind by name.
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self(Cow::Borrowed(name))
    }

    /// The name as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The Prop kind.
pub const PROP: ElementKindName = ElementKindName::new("prop");

/// What the editor knows about an Element kind.
///
/// A kind is a descriptor, not code: adding one never restructures a component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementKindDescriptor {
    /// The kind's name.
    pub name: ElementKindName,
    /// How the kind is shown to the Author.
    pub label: String,
}

/// Every Element kind the editor knows, by name.
#[derive(Resource, Debug, Clone)]
pub struct ElementKindRegistry {
    /// The descriptors, ordered by name.
    kinds: BTreeMap<ElementKindName, ElementKindDescriptor>,
}

impl Default for ElementKindRegistry {
    /// A registry with the built-in kinds: Prop.
    fn default() -> Self {
        let mut registry = Self {
            kinds: BTreeMap::new(),
        };
        registry.register(ElementKindDescriptor {
            name: PROP,
            label: "Prop".to_owned(),
        });
        registry
    }
}

impl ElementKindRegistry {
    /// Adds a kind, replacing a descriptor of the same name.
    pub fn register(&mut self, descriptor: ElementKindDescriptor) {
        self.kinds.insert(descriptor.name.clone(), descriptor);
    }

    /// The descriptor of a kind, if the kind is known.
    #[must_use]
    pub fn get(&self, name: &ElementKindName) -> Option<&ElementKindDescriptor> {
        self.kinds.get(name)
    }

    /// Every known kind, ordered by name.
    pub fn iter(&self) -> impl Iterator<Item = &ElementKindDescriptor> {
        self.kinds.values()
    }
}
