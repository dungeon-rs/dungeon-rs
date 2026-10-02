//! The serialisation registry: how each component is written into a Project file and read back
//! from every version it has had, and the component that keeps what no entry knows.

use crate::{AssetReferences, Bounds, Element, Grid, Layer, Level, Portal, Project, Prop, Wall};
use bevy_ecs::component::Component;
use bevy_ecs::reflect::ReflectComponent;
use bevy_ecs::resource::Resource;
use bevy_ecs::world::{EntityRef, EntityWorldMut};
use bevy_reflect::Reflect;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::collections::BTreeMap;

/// What can go wrong between a component and its envelope.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SerialisationError {
    /// The envelope was written by an editor that knows a newer version of the component.
    #[error(
        "the component `{component}` was saved at version {version}, newer than version {known} this editor knows"
    )]
    NewerVersion {
        /// The component's stable name.
        component: String,
        /// The version in the file.
        version: u32,
        /// The newest version this editor reads.
        known: u32,
    },
    /// The envelope names a version the component never had.
    #[error("the component `{component}` never had a version {version}")]
    UnknownVersion {
        /// The component's stable name.
        component: String,
        /// The version in the file.
        version: u32,
    },
    /// The envelope or its data is not what the component expects.
    #[error("the component `{component}` is malformed: {reason}")]
    Malformed {
        /// The component's stable name.
        component: String,
        /// What is wrong.
        reason: String,
    },
}

/// What a Project file holds for one component of one entity: a version and that version's data.
#[derive(Debug, Serialize, Deserialize)]
pub struct Envelope {
    /// The version of the component's schema the data follows.
    pub version: u32,
    /// The component's data, as written.
    pub data: Box<RawValue>,
}

impl Envelope {
    /// Reads an envelope out of its raw form.
    ///
    /// # Errors
    ///
    /// [`SerialisationError::Malformed`] when the raw value is not an envelope.
    pub fn parse(component: &str, raw: &RawValue) -> Result<Self, SerialisationError> {
        serde_json::from_str(raw.get()).map_err(|error| SerialisationError::Malformed {
            component: component.to_owned(),
            reason: error.to_string(),
        })
    }
}

/// The envelopes of one entity, by stable component name.
pub type Envelopes = BTreeMap<String, Box<RawValue>>;

/// The entity of a Project a component belongs on.
///
/// A Project file keeps each entity's envelopes under its tier, and an envelope under another
/// tier than its component's is malformed: a Level's name on the Project entity would otherwise
/// read as a second Level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// The Project entity.
    Project,
    /// A Level entity.
    Level,
    /// A Layer entity.
    Layer,
    /// An Element entity.
    Element,
}

impl std::fmt::Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Project => "Project",
            Self::Level => "Level",
            Self::Layer => "Layer",
            Self::Element => "Element",
        })
    }
}

/// A component that a Project file can hold.
///
/// Each implementing crate chooses a stable name that never changes once a file has been written
/// with it, names the tier of entity the component belongs on, and bumps the version whenever
/// the data's shape changes; `read` then accepts every version the component has had.
pub trait Serialisable: Component + Serialize + Sized {
    /// The stable name the component is written under.
    const NAME: &'static str;
    /// The version `Serialize` produces.
    const VERSION: u32;
    /// The entity the component belongs on.
    const TIER: Tier;

    /// The component as its data at `version` describes it.
    ///
    /// # Errors
    ///
    /// [`SerialisationError::NewerVersion`] for a version this editor does not know yet,
    /// [`SerialisationError::UnknownVersion`] for one the component never had, or
    /// [`SerialisationError::Malformed`] when the data does not fit the version.
    fn read(version: u32, data: &RawValue) -> Result<Self, SerialisationError>;
}

/// Reads the data of a component that has had one version only, its current one.
///
/// # Errors
///
/// As [`Serialisable::read`].
pub fn read_only_version<C: Serialisable + DeserializeOwned>(
    version: u32,
    data: &RawValue,
) -> Result<C, SerialisationError> {
    if version > C::VERSION {
        return Err(SerialisationError::NewerVersion {
            component: C::NAME.to_owned(),
            version,
            known: C::VERSION,
        });
    }
    if version != C::VERSION {
        return Err(SerialisationError::UnknownVersion {
            component: C::NAME.to_owned(),
            version,
        });
    }
    serde_json::from_str(data.get()).map_err(|error| SerialisationError::Malformed {
        component: C::NAME.to_owned(),
        reason: error.to_string(),
    })
}

/// Envelopes an entity carries that no registered component reads: data written by another
/// version of the editor or by a Plugin, kept verbatim so that saving writes it back unchanged.
///
/// Each envelope is kept as the text it was read as, keyed by its component name.
#[derive(Component, Reflect, Debug, Clone, Default, PartialEq, Eq)]
#[reflect(Component)]
pub struct UnknownComponents {
    /// The raw envelopes, by stable component name.
    pub envelopes: BTreeMap<String, String>,
}

/// How a registered component is written and read.
#[derive(Debug, Clone, Copy)]
pub struct SerialisableComponent {
    /// The stable name.
    pub name: &'static str,
    /// The version written.
    pub version: u32,
    /// The entity the component belongs on.
    pub tier: Tier,
    /// Writes the component of an entity as an envelope, or `None` when the entity has none.
    write: fn(EntityRef) -> Result<Option<Box<RawValue>>, SerialisationError>,
    /// Reads an envelope into the component on an entity.
    read: fn(&mut EntityWorldMut, &RawValue) -> Result<(), SerialisationError>,
}

impl SerialisableComponent {
    /// The entry of a serialisable component.
    fn of<C: Serialisable>() -> Self {
        Self {
            name: C::NAME,
            version: C::VERSION,
            tier: C::TIER,
            write: |entity| {
                let Some(component) = entity.get::<C>() else {
                    return Ok(None);
                };
                let malformed = |error: serde_json::Error| SerialisationError::Malformed {
                    component: C::NAME.to_owned(),
                    reason: error.to_string(),
                };
                let data = serde_json::value::to_raw_value(component).map_err(malformed)?;
                let envelope = Envelope {
                    version: C::VERSION,
                    data,
                };
                serde_json::value::to_raw_value(&envelope)
                    .map(Some)
                    .map_err(malformed)
            },
            read: |entity, raw| {
                let envelope = Envelope::parse(C::NAME, raw)?;
                entity.insert(C::read(envelope.version, &envelope.data)?);
                Ok(())
            },
        }
    }

    /// Writes the component of `entity` as an envelope, or `None` when the entity has none.
    ///
    /// # Errors
    ///
    /// [`SerialisationError::Malformed`] when the component cannot be serialised.
    pub fn write(&self, entity: EntityRef) -> Result<Option<Box<RawValue>>, SerialisationError> {
        (self.write)(entity)
    }

    /// Reads an envelope into the component on `entity`.
    ///
    /// # Errors
    ///
    /// As [`Serialisable::read`].
    pub fn read(
        &self,
        entity: &mut EntityWorldMut,
        raw: &RawValue,
    ) -> Result<(), SerialisationError> {
        (self.read)(entity, raw)
    }
}

/// Every component a Project file can hold, by stable name.
///
/// Each crate registers the components it owns when its plugin is built; the Project lifecycle
/// writes and reads every entity of a Project through it and never names a component itself.
#[derive(Resource, Debug, Clone, Default)]
pub struct SerialisationRegistry {
    /// The entries, by stable name.
    entries: BTreeMap<&'static str, SerialisableComponent>,
}

impl SerialisationRegistry {
    /// Registers a component under its stable name.
    ///
    /// Two components under one name would read each other's envelopes, so registering a name
    /// twice is a programming error, caught in debug builds; a release build keeps the later one.
    pub fn register<C: Serialisable>(&mut self) {
        let previous = self
            .entries
            .insert(C::NAME, SerialisableComponent::of::<C>());
        debug_assert!(
            previous.is_none(),
            "two components are registered under the name `{}`",
            C::NAME
        );
    }

    /// Forgets the component registered under `name`, as an editor that never knew it would, so
    /// its envelopes are kept verbatim from then on. Returns whether a component was registered
    /// under the name.
    ///
    /// The editor itself never forgets a component; tests use this to stand in for an older
    /// editor without it.
    pub fn remove(&mut self, name: &str) -> bool {
        self.entries.remove(name).is_some()
    }

    /// The entry of a component, if its name is registered.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&SerialisableComponent> {
        self.entries.get(name)
    }

    /// Every entry, ordered by name.
    pub fn iter(&self) -> impl Iterator<Item = &SerialisableComponent> {
        self.entries.values()
    }

    /// Checks that no envelope a registered component would read was written at a version newer
    /// than the one registered. Unknown names pass.
    ///
    /// # Errors
    ///
    /// [`SerialisationError::NewerVersion`] naming the component and the version, or
    /// [`SerialisationError::Malformed`] when a known envelope is not one.
    pub fn check_versions(&self, envelopes: &Envelopes) -> Result<(), SerialisationError> {
        for (name, raw) in envelopes {
            let Some(entry) = self.get(name) else {
                continue;
            };
            let envelope = Envelope::parse(name, raw)?;
            if envelope.version > entry.version {
                return Err(SerialisationError::NewerVersion {
                    component: name.clone(),
                    version: envelope.version,
                    known: entry.version,
                });
            }
        }
        Ok(())
    }

    /// Every registered component `entity` carries, as envelopes, together with the envelopes it
    /// carries in [`UnknownComponents`].
    ///
    /// # Errors
    ///
    /// [`SerialisationError::Malformed`] when a component cannot be serialised or a kept
    /// envelope is no longer valid.
    pub fn write_all(&self, entity: EntityRef) -> Result<Envelopes, SerialisationError> {
        let mut envelopes = Envelopes::new();
        if let Some(unknown) = entity.get::<UnknownComponents>() {
            for (name, text) in &unknown.envelopes {
                let raw = RawValue::from_string(text.clone()).map_err(|error| {
                    SerialisationError::Malformed {
                        component: name.clone(),
                        reason: error.to_string(),
                    }
                })?;
                envelopes.insert(name.clone(), raw);
            }
        }
        for entry in self.iter() {
            if let Some(raw) = entry.write(entity)? {
                envelopes.insert(entry.name.to_owned(), raw);
            }
        }
        Ok(envelopes)
    }

    /// Reads every envelope into `entity`, an entity of `tier`: a registered name becomes its
    /// component, and the rest are kept verbatim in [`UnknownComponents`].
    ///
    /// # Errors
    ///
    /// As [`Serialisable::read`], or [`SerialisationError::Malformed`] for a registered
    /// component that belongs on another tier; the entity then holds what was read before the
    /// failure.
    pub fn read_all(
        &self,
        entity: &mut EntityWorldMut,
        envelopes: &Envelopes,
        tier: Tier,
    ) -> Result<(), SerialisationError> {
        let mut unknown = UnknownComponents::default();
        for (name, raw) in envelopes {
            match self.get(name) {
                Some(entry) if entry.tier != tier => {
                    return Err(SerialisationError::Malformed {
                        component: name.clone(),
                        reason: format!(
                            "it belongs on a {} entity, not on a {tier} entity",
                            entry.tier
                        ),
                    });
                }
                Some(entry) => entry.read(entity, raw)?,
                None => {
                    unknown.envelopes.insert(name.clone(), raw.get().to_owned());
                }
            }
        }
        if !unknown.envelopes.is_empty() {
            entity.insert(unknown);
        }
        Ok(())
    }
}

/// Implements [`Serialisable`] for the model's components that have had one version only and
/// need no check beyond their shape, each under its stable name and on its tier, and emits
/// [`register_all`], which registers exactly that list and the components that implement
/// [`Serialisable`] themselves, so a component is never implemented but forgotten or the other
/// way round.
macro_rules! serialisable_at_version_one {
    ($($component:ty => $name:literal on $tier:expr),* $(,)?; checked: $($checked:ty),* $(,)?) => {
        $(
            impl Serialisable for $component {
                const NAME: &'static str = $name;
                const VERSION: u32 = 1;
                const TIER: Tier = $tier;

                fn read(version: u32, data: &RawValue) -> Result<Self, SerialisationError> {
                    read_only_version(version, data)
                }
            }
        )*

        /// Registers every component the model owns.
        pub(crate) fn register_all(registry: &mut SerialisationRegistry) {
            $(registry.register::<$component>();)*
            $(registry.register::<$checked>();)*
        }
    };
}

serialisable_at_version_one! {
    Project => "project" on Tier::Project,
    Grid => "grid" on Tier::Project,
    Bounds => "bounds" on Tier::Project,
    AssetReferences => "asset_references" on Tier::Project,
    Level => "level" on Tier::Level,
    Layer => "layer" on Tier::Layer,
    Element => "element" on Tier::Element,
    Prop => "prop" on Tier::Element;
    checked: Wall, Portal,
}
