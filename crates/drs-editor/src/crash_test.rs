//! Development tooling: a crash forced on purpose, so the crash handler can be seen at work.
//! `DRS_CRASH_TEST` names where: `main` panics on the main thread on the second frame, `thread`
//! on a spawned thread on the second frame, and `startup` while the plugins build, before any
//! window exists.

use bevy::ecs::system::{Local, NonSendMarker};

/// The frame the forced crash happens on: after the first frame has drawn the window.
const FRAME: u32 = 2;

/// Where the crash is forced, from the environment.
fn forced() -> Option<String> {
    std::env::var("DRS_CRASH_TEST").ok()
}

/// Panics while the plugins build when `startup` is asked.
///
/// # Panics
///
/// On purpose, when `DRS_CRASH_TEST` is `startup`.
#[expect(
    clippy::panic,
    clippy::manual_assert,
    reason = "a crash forced on purpose in development"
)]
pub(crate) fn at_startup() {
    if forced().as_deref() == Some("startup") {
        panic!("a crash forced at startup by DRS_CRASH_TEST");
    }
}

/// Panics on the second frame, on the main thread or on a spawned thread, as asked.
///
/// # Panics
///
/// On purpose, when `DRS_CRASH_TEST` is `main` or `thread`.
#[expect(clippy::panic, reason = "a crash forced on purpose in development")]
pub(crate) fn on_second_frame(_main_thread: NonSendMarker, mut frames: Local<u32>) {
    *frames = frames.saturating_add(1);
    if *frames != FRAME {
        return;
    }
    let forced = forced();
    if forced.as_deref() == Some("main") {
        panic!("a crash forced on the main thread by DRS_CRASH_TEST");
    } else if forced.as_deref() == Some("thread") {
        drop(
            std::thread::Builder::new()
                .name("crash test".to_owned())
                .spawn(|| panic!("a crash forced on another thread by DRS_CRASH_TEST")),
        );
    }
}
