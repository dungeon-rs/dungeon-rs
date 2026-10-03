//! The steps of a drag in the viewport: a press becomes a drag once the pointer has travelled a
//! few pixels, and from then on every frame that would send something new sends it as a step of
//! one gesture, so the whole drag is one history step.

use bevy::math::Vec2;
use drs_model::Gesture;

/// How far the pointer travels, in pixels, before a press on an Element, a handle, or a stroke
/// becomes a drag.
pub(crate) const DRAG_THRESHOLD: f32 = 3.0;

/// A press that may become a drag: where the pointer went down, and what the drag last sent, a
/// position or an amount in cells.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Drag {
    /// The pointer, on screen, when the button went down.
    pressed_at: Vec2,
    /// What the last step sent; `None` until the drag begins.
    sent: Option<Vec2>,
}

impl Drag {
    /// A press with the pointer at `cursor`, not yet a drag.
    pub(crate) fn new(cursor: Vec2) -> Self {
        Self {
            pressed_at: cursor,
            sent: None,
        }
    }

    /// The pointer, on screen, when the button went down.
    pub(crate) fn pressed_at(self) -> Vec2 {
        self.pressed_at
    }

    /// Whether the press has become a drag.
    pub(crate) fn begun(self) -> bool {
        self.sent.is_some()
    }

    /// What the last step sent, once the drag has begun.
    pub(crate) fn sent(self) -> Option<Vec2> {
        self.sent
    }

    /// The step that sends `next` with the pointer at `cursor`, if there is one, recorded as
    /// sent: the gesture begins once the pointer has travelled farther than the threshold from
    /// the press, and continues whenever `next` differs from what was last sent, whether the
    /// pointer moved or only what it is shown at.
    pub(crate) fn step(&mut self, cursor: Vec2, next: Vec2) -> Option<Gesture> {
        let gesture = match self.sent {
            None if cursor.distance(self.pressed_at) > DRAG_THRESHOLD => Gesture::Begin,
            Some(sent) if sent != next => Gesture::Continue,
            None | Some(_) => return None,
        };
        self.sent = Some(next);
        Some(gesture)
    }
}
