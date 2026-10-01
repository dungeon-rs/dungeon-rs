//! Development tooling: the editor driven from a script of input steps, one step per frame, so it
//! can be exercised from a terminal that has no pointer or keyboard to lend it.
//!
//! `DRS_SCRIPT` names a plain-text file with one step per line; blank lines and lines starting
//! with `#` are skipped. Positions are logical window pixels with the origin at the top-left
//! corner. The steps are:
//!
//! - `wait <frames>`: do nothing for that many frames.
//! - `move <x> <y>`: move the pointer.
//! - `down <left|middle|right>`, `up <left|middle|right>`: press or release a mouse button.
//! - `click <x> <y>`: move, press, and release the left button over consecutive frames.
//! - `drag <x1> <y1> <x2> <y2> <steps>`: press at the first point, move in that many steps, release.
//! - `key <KeyCode> [cmd] [shift] [ctrl] [alt]`: press a key with the modifiers, release it next frame.
//! - `hold <KeyCode>`, `release <KeyCode>`: press a key and keep it down, or let it go.
//! - `text <string>`: type the characters, one per frame.
//! - `scroll <dx> <dy> [line|pixel]`: scroll the wheel; lines unless told otherwise.
//! - `pinch <delta>`: a trackpad pinch.
//! - `screenshot <path>`: save a screenshot of the window there.
//! - `quit`: exit the editor.
//!
//! Each step becomes the messages the window would have sent, so egui and the Editor's own
//! systems see them alike.

use bevy::app::AppExit;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::query::With;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::{Commands, ResMut, Single, SystemParam};
use bevy::input::ButtonState;
use bevy::input::gestures::PinchGesture;
use bevy::input::keyboard::{Key, KeyCode, KeyboardInput, NativeKey, NativeKeyCode};
use bevy::input::mouse::{MouseButton, MouseButtonInput, MouseScrollUnit, MouseWheel};
use bevy::input::touch::TouchPhase;
use bevy::math::Vec2;
use bevy::reflect::FromReflect;
use bevy::reflect::enums::{DynamicEnum, DynamicVariant};
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::{CursorMoved, PrimaryWindow, Window, WindowEvent};
use std::collections::VecDeque;
use std::path::PathBuf;

/// One thing done to the window.
#[derive(Debug, Clone)]
enum Action {
    /// The pointer moves to a point.
    Move(Vec2),
    /// A mouse button changes state.
    Button(MouseButton, ButtonState),
    /// A key changes state.
    Key {
        /// The physical key.
        code: KeyCode,
        /// The logical key.
        key: Key,
        /// The text the key produces, when it is typed.
        text: Option<String>,
        /// Pressed or released.
        state: ButtonState,
    },
    /// The wheel scrolls.
    Scroll {
        /// Horizontal.
        x: f32,
        /// Vertical.
        y: f32,
        /// Lines or pixels.
        unit: MouseScrollUnit,
    },
    /// A trackpad pinch.
    Pinch(f32),
    /// A screenshot is saved to the path.
    Screenshot(PathBuf),
    /// The editor exits.
    Quit,
}

/// What one frame does: nothing for a while, or a few actions at once.
#[derive(Debug, Clone)]
enum Step {
    /// Idle frames.
    Wait(u32),
    /// The actions of one frame.
    Act(Vec<Action>),
}

/// The script still to run.
#[derive(Resource, Debug)]
pub(crate) struct Script {
    /// The frames to come.
    steps: VecDeque<Step>,
    /// Idle frames left before the next step.
    waiting: u32,
    /// Whether the window has been described in the log.
    described: bool,
}

impl Script {
    /// The script `DRS_SCRIPT` names, or `None` when it is unset or cannot be read or parsed; a
    /// bad script is reported in the log.
    pub(crate) fn from_environment() -> Option<Self> {
        let path = std::env::var_os("DRS_SCRIPT")?;
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) => {
                bevy::log::error!("the script {} cannot be read: {error}", path.display());
                return None;
            }
        };
        match parse(&text) {
            Ok(steps) => Some(Self {
                steps,
                waiting: 0,
                described: false,
            }),
            Err(error) => {
                bevy::log::error!("the script {} is malformed: {error}", path.display());
                None
            }
        }
    }
}

/// The messages a step writes.
#[derive(SystemParam)]
pub(crate) struct Injected<'w> {
    /// The pointer moved.
    cursor: MessageWriter<'w, CursorMoved>,
    /// A mouse button changed.
    buttons: MessageWriter<'w, MouseButtonInput>,
    /// A key changed.
    keys: MessageWriter<'w, KeyboardInput>,
    /// The wheel scrolled.
    wheel: MessageWriter<'w, MouseWheel>,
    /// A pinch.
    pinches: MessageWriter<'w, PinchGesture>,
    /// The same, as the window reports them for egui.
    window_events: MessageWriter<'w, WindowEvent>,
    /// The editor exits.
    exit: MessageWriter<'w, AppExit>,
}

/// Runs the next step of the script, before input is processed so this frame sees it.
pub(crate) fn drive(
    mut script: ResMut<Script>,
    mut window: Single<(Entity, &mut Window), With<PrimaryWindow>>,
    mut commands: Commands,
    mut injected: Injected,
) {
    let (entity, window) = &mut *window;
    if !script.described {
        script.described = true;
        bevy::log::info!(
            "script: the window is {}x{} logical pixels at scale factor {}",
            window.width(),
            window.height(),
            window.scale_factor()
        );
    }
    if script.waiting > 0 {
        script.waiting -= 1;
        return;
    }
    let Some(step) = script.steps.pop_front() else {
        return;
    };
    match step {
        Step::Wait(frames) => script.waiting = frames,
        Step::Act(actions) => {
            for action in actions {
                bevy::log::debug!("script: {action:?}");
                perform(action, *entity, window, &mut commands, &mut injected);
            }
        }
    }
}

/// Does one action to the window.
fn perform(
    action: Action,
    window: Entity,
    primary: &mut Window,
    commands: &mut Commands,
    injected: &mut Injected,
) {
    match action {
        Action::Move(position) => {
            let delta = primary.cursor_position().map(|last| position - last);
            primary.set_cursor_position(Some(position));
            let moved = CursorMoved {
                window,
                position,
                delta,
            };
            injected.cursor.write(moved.clone());
            injected
                .window_events
                .write(WindowEvent::CursorMoved(moved));
        }
        Action::Button(button, state) => {
            let input = MouseButtonInput {
                button,
                state,
                window,
            };
            injected.buttons.write(input);
            injected
                .window_events
                .write(WindowEvent::MouseButtonInput(input));
        }
        Action::Key {
            code,
            key,
            text,
            state,
        } => {
            let input = KeyboardInput {
                key_code: code,
                logical_key: key,
                state,
                text: text.map(Into::into),
                repeat: false,
                window,
            };
            injected.keys.write(input.clone());
            injected
                .window_events
                .write(WindowEvent::KeyboardInput(input));
        }
        Action::Scroll { x, y, unit } => {
            let wheel = MouseWheel {
                unit,
                x,
                y,
                window,
                phase: TouchPhase::Moved,
            };
            injected.wheel.write(wheel);
            injected.window_events.write(WindowEvent::MouseWheel(wheel));
        }
        Action::Pinch(delta) => {
            injected.pinches.write(PinchGesture(delta));
            injected
                .window_events
                .write(WindowEvent::PinchGesture(PinchGesture(delta)));
        }
        Action::Screenshot(path) => {
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(path));
        }
        Action::Quit => {
            injected.exit.write(AppExit::Success);
        }
    }
}

/// Reads a script.
///
/// # Errors
///
/// Names the first line that is not a step.
fn parse(text: &str) -> Result<VecDeque<Step>, String> {
    let mut steps = VecDeque::new();
    for (index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut words = line.split_whitespace();
        let Some(verb) = words.next() else {
            continue;
        };
        let rest: Vec<&str> = words.collect();
        let parsed = match verb {
            "wait" => number(&rest, 0).map(|frames| vec![Step::Wait(frames)]),
            "move" => point(&rest, 0).map(|at| vec![act(Action::Move(at))]),
            "down" => button(&rest).map(|b| vec![act(Action::Button(b, ButtonState::Pressed))]),
            "up" => button(&rest).map(|b| vec![act(Action::Button(b, ButtonState::Released))]),
            "click" => point(&rest, 0).map(click),
            "drag" => drag(&rest),
            "key" => key(&rest),
            "hold" => held(&rest, ButtonState::Pressed),
            "release" => held(&rest, ButtonState::Released),
            "text" => Ok(typed(line.strip_prefix("text").unwrap_or_default().trim())),
            "scroll" => scroll(&rest),
            "pinch" => decimal(&rest, 0).map(|delta| vec![act(Action::Pinch(delta))]),
            "screenshot" => rest
                .first()
                .map(|path| vec![act(Action::Screenshot(PathBuf::from(path)))])
                .ok_or_else(|| "screenshot needs a path".to_owned()),
            "quit" => Ok(vec![act(Action::Quit)]),
            other => Err(format!("unknown step {other}")),
        };
        match parsed {
            Ok(parsed) => steps.extend(parsed),
            Err(error) => return Err(format!("line {}: {error}", index + 1)),
        }
    }
    Ok(steps)
}

/// A frame doing one action.
fn act(action: Action) -> Step {
    Step::Act(vec![action])
}

/// A move, a press, and a release, one per frame.
fn click(at: Vec2) -> Vec<Step> {
    vec![
        act(Action::Move(at)),
        act(Action::Button(MouseButton::Left, ButtonState::Pressed)),
        act(Action::Button(MouseButton::Left, ButtonState::Released)),
    ]
}

/// A press at the first point, moves towards the second, and a release.
///
/// # Errors
///
/// When the two points or the step count are missing or not numbers.
fn drag(words: &[&str]) -> Result<Vec<Step>, String> {
    let from = point(words, 0)?;
    let to = point(words, 2)?;
    let count: u16 = number(words, 4)?;
    let mut steps = vec![
        act(Action::Move(from)),
        act(Action::Button(MouseButton::Left, ButtonState::Pressed)),
    ];
    for i in 1..=count.max(1) {
        let t = f32::from(i) / f32::from(count.max(1));
        steps.push(act(Action::Move(from.lerp(to, t))));
    }
    steps.push(act(Action::Button(
        MouseButton::Left,
        ButtonState::Released,
    )));
    Ok(steps)
}

/// The modifiers, then the key, pressed in one frame and released in reverse the next.
///
/// # Errors
///
/// When the key or a modifier is unknown.
fn key(words: &[&str]) -> Result<Vec<Step>, String> {
    let name = words
        .first()
        .ok_or_else(|| "key needs a KeyCode".to_owned())?;
    let (code, logical) = key_named(name).ok_or_else(|| format!("unknown KeyCode {name}"))?;
    let mut modifiers = Vec::new();
    for word in &words[1..] {
        modifiers.push(match *word {
            "cmd" => (KeyCode::SuperLeft, Key::Super),
            "shift" => (KeyCode::ShiftLeft, Key::Shift),
            "ctrl" => (KeyCode::ControlLeft, Key::Control),
            "alt" => (KeyCode::AltLeft, Key::Alt),
            other => return Err(format!("unknown modifier {other}")),
        });
    }
    let press = |(code, key): &(KeyCode, Key), state| Action::Key {
        code: *code,
        key: key.clone(),
        text: None,
        state,
    };
    let mut down: Vec<Action> = modifiers
        .iter()
        .map(|m| press(m, ButtonState::Pressed))
        .collect();
    down.push(press(&(code, logical.clone()), ButtonState::Pressed));
    let mut up = vec![press(&(code, logical), ButtonState::Released)];
    up.extend(
        modifiers
            .iter()
            .rev()
            .map(|m| press(m, ButtonState::Released)),
    );
    Ok(vec![Step::Act(down), Step::Act(up)])
}

/// A key held down or let go, so another gesture can happen meanwhile.
///
/// # Errors
///
/// When the key is unknown.
fn held(words: &[&str], state: ButtonState) -> Result<Vec<Step>, String> {
    let name = words
        .first()
        .ok_or_else(|| "a KeyCode is needed".to_owned())?;
    let (code, key) = key_named(name).ok_or_else(|| format!("unknown KeyCode {name}"))?;
    Ok(vec![act(Action::Key {
        code,
        key,
        text: None,
        state,
    })])
}

/// Each character typed, pressed and released in its own frame.
fn typed(text: &str) -> Vec<Step> {
    text.chars()
        .map(|c| {
            let character = c.to_string();
            let key = |state| Action::Key {
                code: KeyCode::Unidentified(NativeKeyCode::Unidentified),
                key: Key::Character(character.clone().into()),
                text: Some(character.clone()),
                state,
            };
            Step::Act(vec![key(ButtonState::Pressed), key(ButtonState::Released)])
        })
        .collect()
}

/// A wheel step: lines unless `pixel` is said.
///
/// # Errors
///
/// When the deltas are missing or the unit is unknown.
fn scroll(words: &[&str]) -> Result<Vec<Step>, String> {
    let x = decimal(words, 0)?;
    let y = decimal(words, 1)?;
    let unit = match words.get(2).copied() {
        None | Some("line") => MouseScrollUnit::Line,
        Some("pixel") => MouseScrollUnit::Pixel,
        Some(other) => return Err(format!("unknown unit {other}")),
    };
    Ok(vec![act(Action::Scroll { x, y, unit })])
}

/// The physical and logical key a `KeyCode` variant name stands for: letters and digits type
/// their character, every other key is the logical key of the same name.
fn key_named(name: &str) -> Option<(KeyCode, Key)> {
    let code = KeyCode::from_reflect(&DynamicEnum::new(name, DynamicVariant::Unit))?;
    let character = name
        .strip_prefix("Key")
        .map(str::to_lowercase)
        .or_else(|| name.strip_prefix("Digit").map(str::to_owned))
        .filter(|c| c.chars().count() == 1);
    let logical = match character {
        Some(character) => Key::Character(character.into()),
        None => Key::from_reflect(&DynamicEnum::new(name, DynamicVariant::Unit))
            .unwrap_or(Key::Unidentified(NativeKey::Unidentified)),
    };
    Some((code, logical))
}

/// The mouse button named.
///
/// # Errors
///
/// When the button is unknown.
fn button(words: &[&str]) -> Result<MouseButton, String> {
    match words.first().copied() {
        Some("left") => Ok(MouseButton::Left),
        Some("middle") => Ok(MouseButton::Middle),
        Some("right") => Ok(MouseButton::Right),
        other => Err(format!("unknown button {}", other.unwrap_or_default())),
    }
}

/// Two numbers from a position in the words.
///
/// # Errors
///
/// When either is missing or not a number.
fn point(words: &[&str], at: usize) -> Result<Vec2, String> {
    Ok(Vec2::new(decimal(words, at)?, decimal(words, at + 1)?))
}

/// A decimal number from a position in the words.
///
/// # Errors
///
/// When it is missing or not a number.
fn decimal(words: &[&str], at: usize) -> Result<f32, String> {
    let word = words
        .get(at)
        .ok_or_else(|| "a number is missing".to_owned())?;
    word.parse().map_err(|_| format!("{word} is not a number"))
}

/// A whole number from a position in the words.
///
/// # Errors
///
/// When it is missing or not a whole number.
fn number<N: std::str::FromStr>(words: &[&str], at: usize) -> Result<N, String> {
    let word = words
        .get(at)
        .ok_or_else(|| "a number is missing".to_owned())?;
    word.parse()
        .map_err(|_| format!("{word} is not a whole number"))
}
