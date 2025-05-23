use bevy::ecs::world::CommandQueue;
use bevy::prelude::{warn, BevyError, Commands, Component, Entity, EntityCommands, Query};
use bevy::tasks::futures_lite::future;
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};

/// The [`AsyncCommand`] component provides a way to insert a task that should be merged into ECS
/// at a later time.
///
/// The task in the component will be polled as part of the system scheduling and when completed
/// it will execute it's [`CommandQueue`].
#[derive(Component)]
pub struct AsyncCommand {
    queue: Task<Result<CommandQueue, BevyError>>,
}

impl AsyncCommand {
    /// Spawns an [`AsyncCommand`] component in the given [Commands].
    ///
    /// The result will be executed when the task finishes.
    pub fn spawn(
        commands: &mut Commands,
        future: impl Future<Output = Result<CommandQueue, BevyError>> + Send + 'static,
    ) {
        let command = AsyncCommand::new(future);

        commands.spawn(command);
    }

    #[must_use]
    fn new(future: impl Future<Output = Result<CommandQueue, BevyError>> + Send + 'static) -> Self {
        let task = AsyncComputeTaskPool::get().spawn(future);

        Self { queue: task }
    }
}

/// The system that will attempt to execute the result of all [`AsyncCommand`]s found.
pub(super) fn execute_async_commands(
    mut commands: Commands,
    query: Query<(Entity, &mut AsyncCommand)>,
) {
    for (entity, mut command) in query {
        if let Some(result) = block_on(future::poll_once(&mut command.queue)) {
            match result {
                Ok(mut queue) => commands.append(&mut queue),
                Err(error) => warn!("AsyncCommand failed: {}", error),
            };

            commands.entity(entity).despawn();
        }
    }
}
