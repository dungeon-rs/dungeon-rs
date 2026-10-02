//! A whole Project as its components: what the Project lifecycle gathers from the World to save
//! and materialises into the World on open, in no file's shape.

use crate::{ElementId, Envelopes};
use std::collections::BTreeMap;

/// A whole Project as envelopes: the Project entity's own components, its Levels with their
/// Layers in order, and every Element keyed by its identity.
///
/// Every component is an envelope under its stable name; an Element's kind component sits beside
/// its common `element` envelope. Elements are keyed by identity and each Layer lists its
/// Elements in stacking order, so the same Project always gathers the same snapshot. How the
/// snapshot is laid out in a file, and in which version of that layout, is `ProjectAccess`'s
/// business alone.
#[derive(Debug, Clone, Default)]
pub struct ProjectSnapshot {
    /// The components of the Project entity: the Project, its Grid, its Bounds, and its Asset
    /// Reference table.
    pub project: Envelopes,
    /// The Levels, in order.
    pub levels: Vec<LevelSnapshot>,
    /// Every Element, by identity.
    pub elements: BTreeMap<ElementId, Envelopes>,
}

/// One Level of a [`ProjectSnapshot`].
#[derive(Debug, Clone, Default)]
pub struct LevelSnapshot {
    /// The components of the Level entity.
    pub components: Envelopes,
    /// The Layers, in order.
    pub layers: Vec<LayerSnapshot>,
}

/// One Layer of a [`ProjectSnapshot`].
#[derive(Debug, Clone, Default)]
pub struct LayerSnapshot {
    /// The components of the Layer entity.
    pub components: Envelopes,
    /// The Elements on the Layer in stacking order, the first drawn first.
    pub elements: Vec<ElementId>,
}
