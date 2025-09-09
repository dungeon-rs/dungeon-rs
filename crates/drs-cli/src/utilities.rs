//! Contains utility methods shared across all CLI commands.

use anyhow::bail;
use bevy::ecs::world::CommandQueue;
use bevy::prelude::{Event, Events, World};
use drs_utils::Sender;
use std::thread::JoinHandle;
use std::time::Duration;

use std::rc::Rc;

/// Utility method that generates a background thread that tracks progress and completion events
/// over the given `Sender<CommandQueue>`.
///
/// You can pass in arguments required within the closures through `TContext`.
///
/// This method allows calling `AsyncComponent` compatible methods from the CLI.
///
/// # Panics
/// This method may cause panics when it fails to resolve the resources it needs to read events.
pub fn track_progress<TProgress, TFProgress, TComplete, TFComplete, TError, TFError, TContext>(
    on_progress: TFProgress,
    on_complete: TFComplete,
    on_error: TFError,
    context: TContext,
    timeout: Duration,
) -> (Sender<CommandQueue>, JoinHandle<Result<(), anyhow::Error>>)
where
    TProgress: Event,
    TFProgress: Fn(TProgress, Rc<TContext>) -> anyhow::Result<()> + Send + 'static,
    TComplete: Event,
    TFComplete: FnOnce(TComplete, Rc<TContext>) -> anyhow::Result<()> + Send + 'static,
    TError: Event,
    TFError: Fn(TError, Rc<TContext>) -> anyhow::Result<()> + Send + 'static,
    TContext: Send + 'static,
{
    let (sender, receiver) = drs_utils::command_queue();

    let thread = std::thread::spawn(move || {
        let mut world = World::default();
        world.init_resource::<Events<TProgress>>();
        world.init_resource::<Events<TComplete>>();
        world.init_resource::<Events<TError>>();

        let context = Rc::new(context);

        loop {
            let mut queue = match receiver.recv_timeout(timeout) {
                Ok(queue) => queue,
                Err(error) => bail!("Timeout while waiting for events: {}", error),
            };

            queue.apply(&mut world);

            // Process progress events
            let mut progress_events = world
                .get_resource_mut::<Events<TProgress>>()
                .expect("Failed to get progress events");

            for event in progress_events.drain() {
                on_progress(event, Rc::clone(&context))?;
            }

            // Process error events
            let mut error_events = world
                .get_resource_mut::<Events<TError>>()
                .expect("Failed to get error events");

            for event in error_events.drain() {
                on_error(event, Rc::clone(&context))?;
            }

            // Process completion events
            let mut completed_events = world
                .get_resource_mut::<Events<TComplete>>()
                .expect("Failed to get completed events");

            if let Some(event) = completed_events.drain().next() {
                return on_complete(event, context);
            }
        }
    });

    (sender, thread)
}
