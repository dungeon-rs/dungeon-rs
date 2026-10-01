//! Generic commands driven by reflection, so most properties need no command of their own.

use crate::{HistoryError, ReversibleCommand};
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::error::BevyError;
use bevy_ecs::reflect::{AppTypeRegistry, ReflectComponent};
use bevy_ecs::world::World;
use bevy_reflect::{GetPath, ParsedPath, PartialReflect, Reflect, TypePath, TypeRegistry};
use core::any::TypeId;

/// Names the entity a generic command works on.
///
/// Keying a command by a stable identity instead of an entity handle lets it survive the entity
/// being despawned and respawned by other steps. An [`Entity`] itself is a `Target` for cases
/// where no step respawns it.
pub trait Target: Send + Sync + 'static {
    /// The entity the target currently names.
    ///
    /// # Errors
    ///
    /// [`HistoryError::MissingTarget`] when no entity carries the identity.
    fn entity(&self, world: &mut World) -> Result<Entity, BevyError>;
}

impl Target for Entity {
    fn entity(&self, world: &mut World) -> Result<Entity, BevyError> {
        world
            .get_entity(*self)
            .map(|_| *self)
            .map_err(|_| HistoryError::MissingTarget.into())
    }
}

/// Finds the `ReflectComponent` type data of a component type.
///
/// # Errors
///
/// [`HistoryError::UnregisteredComponent`] when the type has none.
fn reflect_component<'r>(
    registry: &'r TypeRegistry,
    component: TypeId,
    type_path: &str,
) -> Result<&'r ReflectComponent, HistoryError> {
    registry
        .get(component)
        .and_then(|registration| registration.data::<ReflectComponent>())
        .ok_or_else(|| HistoryError::UnregisteredComponent(type_path.to_owned()))
}

/// Sets one field of one component by reflect path, remembering the value it replaces.
///
/// Applying it again after a revert sets the same value; reverting restores the value the field
/// had before the first application.
pub struct SetField<T: Target> {
    /// Which entity.
    target: T,
    /// Which component.
    component: TypeId,
    /// The component's type path, for error messages.
    type_path: &'static str,
    /// Which field.
    path: ParsedPath,
    /// The path as written, for error messages.
    path_text: String,
    /// The value to set.
    value: Box<dyn PartialReflect>,
    /// The value the field had before the first application.
    previous: Option<Box<dyn PartialReflect>>,
}

impl<T: Target> SetField<T> {
    /// A command that sets `path` inside component `C` of `target` to `value`.
    ///
    /// # Errors
    ///
    /// [`HistoryError::Path`] when the path cannot be parsed.
    pub fn new<C: Component + Reflect + TypePath>(
        target: T,
        path: &str,
        value: impl PartialReflect,
    ) -> Result<Self, HistoryError> {
        let parsed = ParsedPath::parse(path).map_err(|error| HistoryError::Path {
            path: path.to_owned(),
            reason: error.to_string(),
        })?;
        Ok(Self {
            target,
            component: TypeId::of::<C>(),
            type_path: C::type_path(),
            path: parsed,
            path_text: path.to_owned(),
            value: Box::new(value),
            previous: None,
        })
    }

    /// Writes `value` into the field, returning the value it replaced.
    ///
    /// # Errors
    ///
    /// When the target, its component, or the field cannot be found, or the value does not fit.
    fn swap(
        &self,
        world: &mut World,
        value: &dyn PartialReflect,
    ) -> Result<Box<dyn PartialReflect>, BevyError> {
        let entity = self.target.entity(world)?;
        let registry = world.resource::<AppTypeRegistry>().clone();
        let registry = registry.read();
        let reflect = reflect_component(&registry, self.component, self.type_path)?;
        let mut entity = world
            .get_entity_mut(entity)
            .map_err(|_| HistoryError::MissingTarget)?;
        let mut component = reflect
            .reflect_mut(&mut entity)
            .ok_or_else(|| HistoryError::MissingComponent(self.type_path.to_owned()))?;
        let field = component
            .reflect_path_mut(&self.path)
            .map_err(|error| HistoryError::Path {
                path: self.path_text.clone(),
                reason: error.to_string(),
            })?;
        let previous = field.to_dynamic()?;
        field.try_apply(value)?;
        Ok(previous)
    }
}

impl<T: Target> ReversibleCommand for SetField<T> {
    fn apply(&mut self, world: &mut World) -> Result<(), BevyError> {
        let previous = self.swap(world, self.value.as_ref())?;
        if self.previous.is_none() {
            self.previous = Some(previous);
        }
        Ok(())
    }

    fn revert(&mut self, world: &mut World) -> Result<(), BevyError> {
        let Some(previous) = &self.previous else {
            return Ok(());
        };
        self.swap(world, previous.as_ref())?;
        Ok(())
    }
}

/// Removes an entity, keeping a snapshot of every reflected component so that it can be restored.
///
/// Only components registered with `ReflectComponent` type data are kept; anything else is lost
/// with the entity. Reverting spawns a new entity carrying the snapshot, available through
/// [`restored`](Self::restored); relationships such as a parent are restored with it, appended
/// to the parent's children.
pub struct Snapshot<T: Target> {
    /// Which entity.
    target: T,
    /// The reflected components, taken when the entity was removed.
    components: Vec<Box<dyn PartialReflect>>,
    /// The entity the last revert spawned.
    restored: Option<Entity>,
}

impl<T: Target> Snapshot<T> {
    /// A command that removes `target` when applied and restores it when reverted.
    pub fn new(target: T) -> Self {
        Self {
            target,
            components: Vec::new(),
            restored: None,
        }
    }

    /// The entity the last revert spawned, if any.
    #[must_use]
    pub fn restored(&self) -> Option<Entity> {
        self.restored
    }
}

impl<T: Target> ReversibleCommand for Snapshot<T> {
    fn apply(&mut self, world: &mut World) -> Result<(), BevyError> {
        let entity = self.target.entity(world)?;
        let registry = world.resource::<AppTypeRegistry>().clone();
        let registry = registry.read();
        let entity_ref = world
            .get_entity(entity)
            .map_err(|_| HistoryError::MissingTarget)?;
        let mut components = Vec::new();
        for id in entity_ref.archetype().components() {
            let Some(type_id) = world
                .components()
                .get_info(*id)
                .and_then(bevy_ecs::component::ComponentInfo::type_id)
            else {
                continue;
            };
            let Some(reflect) = registry
                .get(type_id)
                .and_then(|registration| registration.data::<ReflectComponent>())
            else {
                continue;
            };
            if let Some(component) = reflect.reflect(entity_ref) {
                components.push(component.to_dynamic()?);
            }
        }
        drop(registry);
        self.components = components;
        world.despawn(entity);
        self.restored = None;
        Ok(())
    }

    fn revert(&mut self, world: &mut World) -> Result<(), BevyError> {
        let registry = world.resource::<AppTypeRegistry>().clone();
        let registry = registry.read();
        let mut entity = world.spawn_empty();
        for component in &self.components {
            let Some(info) = component.get_represented_type_info() else {
                continue;
            };
            let reflect = reflect_component(&registry, info.type_id(), info.type_path())?;
            reflect.insert(&mut entity, component.as_ref(), &registry);
        }
        self.restored = Some(entity.id());
        Ok(())
    }
}
