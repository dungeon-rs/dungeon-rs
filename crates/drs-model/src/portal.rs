//! Portals: images with a width, freestanding or set into a Wall or a Room's Walls, and whether
//! each follows the host its anchor names.

use crate::{
    AssetReferenceRow, ElementId, ElementKindName, Serialisable, SerialisationError, Tier,
    read_current_version,
};
use bevy_ecs::component::Component;
use bevy_ecs::reflect::ReflectComponent;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

/// The Portal kind.
pub const PORTAL: ElementKindName = ElementKindName::new("portal");

/// Which side of a segment a Portal set into it faces, looking along the segment from its first
/// point to its second.
#[derive(Reflect, Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Side {
    /// The left of the segment's direction: the Portal's image is drawn as it is.
    #[default]
    Left,
    /// The right of the segment's direction: the Portal's image is mirrored across the line.
    Right,
}

impl Side {
    /// The other side.
    #[must_use]
    pub const fn flipped(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    /// The side a Portal faces when its image is mirrored or not: the right when mirrored.
    #[must_use]
    pub const fn of_mirroring(mirrored: bool) -> Self {
        if mirrored { Self::Right } else { Self::Left }
    }

    /// Whether a Portal facing this side is drawn mirrored.
    #[must_use]
    pub const fn mirrors(self) -> bool {
        matches!(self, Self::Right)
    }
}

/// Where a Portal is set: the Element it is set into, the number of the part of that Element it
/// is set into, the parameter along that part from zero at its first point to one at its second,
/// and the side it faces.
///
/// The anchor names the Element by its identity, so it survives saving, undo, and redo; only the
/// edits that renumber the segments move it to another segment.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PortalAnchor {
    /// The Element the Portal is set into.
    pub host: ElementId,
    /// Which part of the Element the Portal is set into, counted from zero: the segment of a Wall
    /// or the edge of a Room.
    pub index: usize,
    /// The parameter along that part, from zero to one.
    pub t: f32,
    /// The side the Portal faces.
    pub side: Side,
}

/// An opening such as a door or a window: an image shown at a width in Grid cells, its height
/// keeping the image's proportions, either freestanding or set into a Wall.
///
/// The Element's position is the Portal's centre and its size the width by that height. A
/// freestanding Portal stands at its position, turned by its rotation and mirrored or not; a set
/// Portal follows its anchor, and the authoring Manager keeps its position, rotation, and
/// mirroring equal to what the anchor gives, so freeing it is clearing the anchor and nothing
/// else.
#[derive(Component, Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Portal {
    /// The row of the Project's Asset Reference table that names the image.
    pub asset: AssetReferenceRow,
    /// How wide the Portal is, in Grid cells, along its Wall when set into one.
    pub width: f32,
    /// How far the Portal is turned counter-clockwise, in radians.
    pub rotation: f32,
    /// Whether the image is mirrored across the Portal's length.
    pub mirrored: bool,
    /// Where the Portal is set, or `None` for a freestanding one.
    pub anchor: Option<PortalAnchor>,
}

impl Portal {
    /// Whether the Portal follows the Wall or the Room its anchor names, as `anchoring` says:
    /// a Portal whose anchoring is not derived yet follows its anchor if it has one, as it will
    /// once it is derived.
    #[must_use]
    pub fn follows(&self, anchoring: Option<&Anchoring>) -> bool {
        anchoring.map_or(self.anchor.is_some(), |anchoring| {
            *anchoring == Anchoring::Set
        })
    }

    /// Why the Portal is not one, if it is not: a width not above zero, a rotation that is not
    /// finite, or an anchor whose parameter is not between zero and one.
    #[must_use]
    pub fn malformation(&self) -> Option<String> {
        if !(self.width > 0.0 && self.width.is_finite()) {
            return Some(format!(
                "a Portal's width must be above zero, not {}",
                self.width
            ));
        }
        if !self.rotation.is_finite() {
            return Some(format!(
                "a Portal's rotation must be finite, not {}",
                self.rotation
            ));
        }
        if let Some(anchor) = &self.anchor
            && !(0.0..=1.0).contains(&anchor.t)
        {
            return Some(format!(
                "a Portal is set at a parameter from 0 to 1 along its segment, not {}",
                anchor.t
            ));
        }
        None
    }
}

impl Serialisable for Portal {
    const NAME: &'static str = "portal";
    const VERSION: u32 = 1;
    const TIER: Tier = Tier::Element;

    fn read(version: u32, data: &RawValue) -> Result<Self, SerialisationError> {
        let portal: Self = read_current_version(version, data)?;
        match portal.malformation() {
            Some(reason) => Err(SerialisationError::Malformed {
                component: Self::NAME.to_owned(),
                reason,
            }),
            None => Ok(portal),
        }
    }
}

/// Whether a Portal stands where its anchor puts it, derived from its anchor and the Wall or the
/// Room it names. It is never saved; the authoring Manager derives it whenever a Portal or a Wall
/// or Room changes, and whoever must tell a Portal that follows its host from one that stands on
/// its own reads it.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Anchoring {
    /// The Portal has no anchor: it stands, turns, and mirrors where it is put.
    #[default]
    Freestanding,
    /// Its anchor names a Wall or a Room on the Portal's Level and a segment or edge it has: the
    /// Portal stands on that part at its parameter, turned to the line, and faces its side.
    Set,
    /// Its anchor names no Wall or Room of its Level, or a part its host lacks, as an editor that
    /// does not know Portals or Rooms may leave it: the Portal keeps its anchor but stands, turns,
    /// and mirrors as a freestanding one.
    Lost,
}
