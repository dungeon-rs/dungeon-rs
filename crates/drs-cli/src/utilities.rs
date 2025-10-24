//! Contains utility methods shared across all CLI commands.

use anyhow::bail;
use bevy::ecs::world::CommandQueue;
use bevy::prelude::{Message, Messages, World};
use drs_utils::Sender;
use std::thread::JoinHandle;
use std::time::Duration;

use std::rc::Rc;

/// Utility method that generates a background thread that tracks progress and completion messages
/// over the given `Sender<CommandQueue>`.
///
/// You can pass in arguments required within the closures through `TContext`.
///
/// This method allows calling `AsyncComponent` compatible methods from the CLI.
///
/// # Panics
/// This method may cause panics when it fails to resolve the resources it needs to read messages.
pub fn track_progress<TProgress, TFProgress, TComplete, TFComplete, TError, TFError, TContext>(
    on_progress: TFProgress,
    on_complete: TFComplete,
    on_error: TFError,
    context: TContext,
    timeout: Duration,
) -> (Sender<CommandQueue>, JoinHandle<Result<(), anyhow::Error>>)
where
    TProgress: Message,
    TFProgress: Fn(TProgress, Rc<TContext>) -> anyhow::Result<()> + Send + 'static,
    TComplete: Message,
    TFComplete: FnOnce(TComplete, Rc<TContext>) -> anyhow::Result<()> + Send + 'static,
    TError: Message,
    TFError: Fn(TError, Rc<TContext>) -> anyhow::Result<()> + Send + 'static,
    TContext: Send + 'static,
{
    let (sender, receiver) = drs_utils::command_queue();

    let thread = std::thread::spawn(move || {
        let mut world = World::default();
        world.init_resource::<Messages<TProgress>>();
        world.init_resource::<Messages<TComplete>>();
        world.init_resource::<Messages<TError>>();

        let context = Rc::new(context);

        loop {
            let mut queue = match receiver.recv_timeout(timeout) {
                Ok(queue) => queue,
                Err(error) => bail!("Timeout while waiting for messages: {error}"),
            };

            queue.apply(&mut world);

            // Process progress messages
            let mut progress_messages = world
                .get_resource_mut::<Messages<TProgress>>()
                .expect("Failed to get progress messages");

            for message in progress_messages.drain() {
                on_progress(message, Rc::clone(&context))?;
            }

            // Process error messages
            let mut error_messages = world
                .get_resource_mut::<Messages<TError>>()
                .expect("Failed to get error messages");

            for message in error_messages.drain() {
                on_error(message, Rc::clone(&context))?;
            }

            // Process completion messages
            let mut completed_messages = world
                .get_resource_mut::<Messages<TComplete>>()
                .expect("Failed to get completed messages");

            if let Some(message) = completed_messages.drain().next() {
                return on_complete(message, context);
            }
        }
    });

    (sender, thread)
}
