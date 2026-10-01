//! The keyboard shortcuts, stated once so that the menu shows exactly what the keys do.

use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use egui::{Key, KeyboardShortcut, Modifiers};

/// One shortcut: how egui spells it for the menu, and the physical key the window reports.
pub(crate) struct Binding {
    /// The shortcut as the menu shows it; `Modifiers::COMMAND` is Command on macOS and Control
    /// elsewhere.
    shortcut: KeyboardShortcut,
    /// The physical key.
    code: KeyCode,
}

impl Binding {
    /// A shortcut of `key` with `modifiers`, reported by the window as `code`.
    const fn new(modifiers: Modifiers, key: Key, code: KeyCode) -> Self {
        Self {
            shortcut: KeyboardShortcut::new(modifiers, key),
            code,
        }
    }

    /// Whether the key went down this frame with exactly the shortcut's modifiers held.
    fn pressed(&self, keys: &ButtonInput<KeyCode>) -> bool {
        let command = if cfg!(target_os = "macos") {
            keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight])
        } else {
            keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
        };
        let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
        let alt = keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);
        keys.just_pressed(self.code)
            && command == self.shortcut.modifiers.command
            && shift == self.shortcut.modifiers.shift
            && alt == self.shortcut.modifiers.alt
    }
}

/// Open…: the platform's usual shortcut.
pub(crate) const OPEN: &[Binding] = &[Binding::new(Modifiers::COMMAND, Key::O, KeyCode::KeyO)];

/// Save: the platform's usual shortcut.
pub(crate) const SAVE: &[Binding] = &[Binding::new(Modifiers::COMMAND, Key::S, KeyCode::KeyS)];

/// Save As…: the platform's usual shortcut.
pub(crate) const SAVE_AS: &[Binding] = &[Binding::new(
    Modifiers::COMMAND.plus(Modifiers::SHIFT),
    Key::S,
    KeyCode::KeyS,
)];

/// Export Level…
pub(crate) const EXPORT: &[Binding] = &[Binding::new(Modifiers::COMMAND, Key::E, KeyCode::KeyE)];

/// Quit: the platform's usual shortcut on macOS and Linux; on Windows the window's own Alt+F4
/// is the way, so the entry has none.
#[cfg(not(target_os = "windows"))]
pub(crate) const QUIT: &[Binding] = &[Binding::new(Modifiers::COMMAND, Key::Q, KeyCode::KeyQ)];

/// Quit: no shortcut of the editor's own on Windows.
#[cfg(target_os = "windows")]
pub(crate) const QUIT: &[Binding] = &[];

/// Undo: the platform's usual shortcut.
pub(crate) const UNDO: &[Binding] = &[Binding::new(Modifiers::COMMAND, Key::Z, KeyCode::KeyZ)];

/// Redo: the platform's usual shortcut.
#[cfg(target_os = "macos")]
pub(crate) const REDO: &[Binding] = &[Binding::new(
    Modifiers::COMMAND.plus(Modifiers::SHIFT),
    Key::Z,
    KeyCode::KeyZ,
)];

/// Redo: the platform's usual shortcuts, either of which works.
#[cfg(not(target_os = "macos"))]
pub(crate) const REDO: &[Binding] = &[
    Binding::new(
        Modifiers::COMMAND.plus(Modifiers::SHIFT),
        Key::Z,
        KeyCode::KeyZ,
    ),
    Binding::new(Modifiers::COMMAND, Key::Y, KeyCode::KeyY),
];

/// Remove the selected Prop: Delete, and on macOS Backspace too, as its keyboards have no Delete.
#[cfg(target_os = "macos")]
pub(crate) const REMOVE: &[Binding] = &[
    Binding::new(Modifiers::NONE, Key::Delete, KeyCode::Delete),
    Binding::new(Modifiers::NONE, Key::Backspace, KeyCode::Backspace),
];

/// Remove the selected Prop: Delete.
#[cfg(not(target_os = "macos"))]
pub(crate) const REMOVE: &[Binding] =
    &[Binding::new(Modifiers::NONE, Key::Delete, KeyCode::Delete)];

/// Whether any of the bindings was pressed this frame.
pub(crate) fn any_pressed(bindings: &[Binding], keys: &ButtonInput<KeyCode>) -> bool {
    bindings.iter().any(|binding| binding.pressed(keys))
}

/// The bindings as the menu shows them next to their entry.
pub(crate) fn shortcut_text(ctx: &egui::Context, bindings: &[Binding]) -> String {
    bindings
        .iter()
        .map(|binding| ctx.format_shortcut(&binding.shortcut))
        .collect::<Vec<_>>()
        .join(", ")
}
