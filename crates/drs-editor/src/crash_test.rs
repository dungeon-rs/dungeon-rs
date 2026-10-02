//! Development tooling: a crash forced on purpose, so the crash handler can be seen at work.
//! `DRS_CRASH_TEST` names where: `main` panics on the main thread on the second frame, `thread`
//! on a spawned thread on the second frame, and `startup` while the plugins build, before any
//! window exists.

use bevy::ecs::resource::Resource;
use bevy::ecs::system::{Local, NonSendMarker, Res};

/// The frame the forced crash happens on: after the first frame has drawn the window.
const FRAME: u32 = 2;

/// Where a crash is forced on the second frame, as `DRS_CRASH_TEST` named it when the plugins
/// built.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ForcedCrash {
    /// On the main thread.
    Main,
    /// On a spawned thread.
    Thread,
}

impl ForcedCrash {
    /// Reads `DRS_CRASH_TEST` once, while the plugins build: `startup` panics here and now,
    /// before any window exists; `main` and `thread` wait for the second frame; anything else
    /// forces nothing and is logged.
    ///
    /// # Panics
    ///
    /// On purpose, when `DRS_CRASH_TEST` is `startup`.
    #[expect(clippy::panic, reason = "a crash forced on purpose in development")]
    pub(crate) fn from_environment() -> Option<Self> {
        match std::env::var("DRS_CRASH_TEST").ok()?.as_str() {
            "startup" => panic!("a crash forced at startup by DRS_CRASH_TEST"),
            "main" => Some(Self::Main),
            "thread" => Some(Self::Thread),
            other => {
                bevy::log::warn!("DRS_CRASH_TEST names no crash to force: {other}");
                None
            }
        }
    }
}

/// Panics on the second frame, on the main thread or on a spawned thread, as asked.
///
/// # Panics
///
/// On purpose, as `DRS_CRASH_TEST` asked.
#[expect(clippy::panic, reason = "a crash forced on purpose in development")]
pub(crate) fn on_second_frame(
    _main_thread: NonSendMarker,
    forced: Res<ForcedCrash>,
    mut frames: Local<u32>,
) {
    *frames = frames.saturating_add(1);
    if *frames != FRAME {
        return;
    }
    match *forced {
        ForcedCrash::Main => panic!("a crash forced on the main thread by DRS_CRASH_TEST"),
        ForcedCrash::Thread => {
            let spawned = std::thread::Builder::new()
                .name("crash test".to_owned())
                .spawn(|| panic!("a crash forced on another thread by DRS_CRASH_TEST"));
            if let Err(error) = spawned {
                bevy::log::error!("the thread for the forced crash could not be started: {error}");
            }
        }
    }
}
