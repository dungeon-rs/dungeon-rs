//! The undo and redo stacks and the operations that move along them.

use crate::{HistoryError, ReversibleCommand};
use bevy_app::{App, Plugin};
use bevy_ecs::error::BevyError;
use bevy_ecs::resource::Resource;
use bevy_ecs::world::World;

/// A recorded command.
type Boxed = Box<dyn ReversibleCommand>;

/// A step as recorded, with the number that tells it apart from every other step ever recorded.
struct Step {
    /// The step's number: fresh for every recorded step, kept through undo and redo.
    sequence: u64,
    /// What the step does.
    commands: Commands,
}

/// One undoable step: a single command or a group undone as one.
enum Commands {
    /// A command recorded on its own.
    Single(Boxed),
    /// Commands recorded while a group was open, in the order they were applied.
    Group(Vec<Boxed>),
}

impl Step {
    /// Reverts the step, last command first. When a command of a group cannot be reverted,
    /// the commands after it are applied again, so the step stays whole where it was.
    ///
    /// # Errors
    ///
    /// The first command that cannot be reverted.
    fn revert(&mut self, world: &mut World) -> Result<(), BevyError> {
        match &mut self.commands {
            Commands::Single(command) => command.revert(world),
            Commands::Group(commands) => {
                for index in (0..commands.len()).rev() {
                    if let Err(error) = commands[index].revert(world) {
                        reapply(&mut commands[index + 1..], world);
                        return Err(error);
                    }
                }
                Ok(())
            }
        }
    }

    /// Applies the step again, first command first. When a command of a group cannot be
    /// applied, the commands before it are reverted again, so the step stays undone whole.
    ///
    /// # Errors
    ///
    /// The first command that cannot be applied.
    fn apply(&mut self, world: &mut World) -> Result<(), BevyError> {
        match &mut self.commands {
            Commands::Single(command) => command.apply(world),
            Commands::Group(commands) => {
                for index in 0..commands.len() {
                    if let Err(error) = commands[index].apply(world) {
                        take_back(&mut commands[..index], world);
                        return Err(error);
                    }
                }
                Ok(())
            }
        }
    }
}

/// Applies `commands` again, first first, after a later command of their group could not be
/// reverted; it stops at the first that cannot be, which leaves nothing better to do.
fn reapply(commands: &mut [Boxed], world: &mut World) {
    for command in commands {
        if command.apply(world).is_err() {
            return;
        }
    }
}

/// Reverts `commands`, last first, after a later command of their group could not be applied;
/// it stops at the first that cannot be, which leaves nothing better to do.
fn take_back(commands: &mut [Boxed], world: &mut World) {
    for command in commands.iter_mut().rev() {
        if command.revert(world).is_err() {
            return;
        }
    }
}

/// Where the history stands: the same value exactly when the history stands at the same recorded
/// step, so a position kept at one moment tells later whether anything has changed since.
///
/// Undoing back to a remembered position compares equal to it; recording a new step never
/// repeats an earlier position, however many steps are undone first. Clearing the history puts
/// it back at the position of an empty history, so a position is compared only with positions
/// taken since the last [`clear`](History::clear), as a Project opened from a file starts afresh.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Position(u64);

/// The history of applied commands: what can be undone and what can be redone.
#[derive(Resource, Default)]
pub struct History {
    /// Steps that can be undone, the most recent last.
    undo: Vec<Step>,
    /// Steps that were undone and can be redone, the next one last.
    redo: Vec<Step>,
    /// The group being recorded, while one is open: its number and its commands.
    group: Option<(u64, Vec<Boxed>)>,
    /// The number the next recorded step gets; never reused, not even after [`clear`](Self::clear).
    next_sequence: u64,
}

impl History {
    /// How many steps can be undone.
    #[must_use]
    pub fn undo_depth(&self) -> usize {
        self.undo.len() + usize::from(self.group.as_ref().is_some_and(|(_, g)| !g.is_empty()))
    }

    /// Where the history stands now: the position of the most recent step that can be undone,
    /// a gesture in progress included, or the position of an empty history.
    #[must_use]
    pub fn position(&self) -> Position {
        if let Some((sequence, commands)) = &self.group
            && !commands.is_empty()
        {
            return Position(*sequence);
        }
        Position(self.undo.last().map_or(0, |step| step.sequence))
    }

    /// Forgets every step, undone or not, and any group in progress. The history then stands at
    /// the position of an empty history again, and the steps recorded afterwards take positions
    /// no earlier step had.
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.group = None;
    }

    /// A step number no earlier step had.
    fn next_sequence(&mut self) -> u64 {
        self.next_sequence += 1;
        self.next_sequence
    }

    /// Whether there is a step to undo.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.undo_depth() > 0
    }

    /// Whether there is a step to redo.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Starts a group: every command applied until [`end_group`](Self::end_group) is one step.
    ///
    /// Opening a group while one is open closes the open one first.
    pub fn begin_group(&mut self) {
        self.end_group();
        let sequence = self.next_sequence();
        self.group = Some((sequence, Vec::new()));
    }

    /// Closes the open group, if any, making its commands one step. An empty group leaves no step.
    pub fn end_group(&mut self) {
        if let Some((sequence, commands)) = self.group.take()
            && !commands.is_empty()
        {
            self.undo.push(Step {
                sequence,
                commands: Commands::Group(commands),
            });
        }
    }

    /// Records an applied command and discards whatever could be redone.
    fn record(&mut self, command: Boxed) {
        self.redo.clear();
        if let Some((_, group)) = &mut self.group {
            group.push(command);
        } else {
            let sequence = self.next_sequence();
            self.undo.push(Step {
                sequence,
                commands: Commands::Single(command),
            });
        }
    }
}

/// Registers the [`History`] resource.
pub struct HistoryPlugin;

impl Plugin for HistoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<History>();
    }
}

/// Carries a command out and records it as a step (or as part of the open group).
///
/// # Errors
///
/// The command's own error when it cannot be applied, in which case nothing is recorded, or
/// [`HistoryError::NoHistory`] when the `World` has no [`History`].
pub fn apply(world: &mut World, mut command: impl ReversibleCommand) -> Result<(), BevyError> {
    if !world.contains_resource::<History>() {
        return Err(HistoryError::NoHistory.into());
    }
    command.apply(world)?;
    world
        .get_resource_mut::<History>()
        .ok_or(HistoryError::NoHistory)?
        .record(Box::new(command));
    Ok(())
}

/// Carries a command out and records it as a step of its own: a group left open by a gesture is
/// closed first, so the command never joins it.
///
/// # Errors
///
/// As [`apply`].
pub fn apply_step(world: &mut World, command: impl ReversibleCommand) -> Result<(), BevyError> {
    world
        .get_resource_mut::<History>()
        .ok_or(HistoryError::NoHistory)?
        .end_group();
    apply(world, command)
}

/// Takes back every command of the open group, last first, and forgets the group, so a step
/// whose later command failed leaves the World as it was before the group began, and nothing is
/// recorded. With no group open it does nothing.
///
/// # Errors
///
/// The first command that cannot be reverted; the commands before it in the group stay applied
/// and are forgotten with it. [`HistoryError::NoHistory`] when the `World` has no [`History`].
pub fn abandon_group(world: &mut World) -> Result<(), BevyError> {
    if !world.contains_resource::<History>() {
        return Err(HistoryError::NoHistory.into());
    }
    world.resource_scope(|world, mut history: bevy_ecs::world::Mut<History>| {
        let Some((_, mut commands)) = history.group.take() else {
            return Ok(());
        };
        commands
            .iter_mut()
            .rev()
            .try_for_each(|command| command.revert(world))
    })
}

/// Takes the most recent step back. Returns `false` when there was nothing to undo.
///
/// An open group is closed first, so a gesture in progress is undone as one step.
///
/// # Errors
///
/// The command's error when it cannot be reverted; the step then stays on the undo stack.
/// [`HistoryError::NoHistory`] when the `World` has no [`History`].
pub fn undo(world: &mut World) -> Result<bool, BevyError> {
    if !world.contains_resource::<History>() {
        return Err(HistoryError::NoHistory.into());
    }
    world.resource_scope(|world, mut history: bevy_ecs::world::Mut<History>| {
        history.end_group();
        let Some(mut step) = history.undo.pop() else {
            return Ok(false);
        };
        match step.revert(world) {
            Ok(()) => {
                history.redo.push(step);
                Ok(true)
            }
            Err(error) => {
                history.undo.push(step);
                Err(error)
            }
        }
    })
}

/// Carries the most recently undone step out again. Returns `false` when there was nothing to redo.
///
/// # Errors
///
/// The command's error when it cannot be applied again; the step then stays on the redo stack.
/// [`HistoryError::NoHistory`] when the `World` has no [`History`].
pub fn redo(world: &mut World) -> Result<bool, BevyError> {
    if !world.contains_resource::<History>() {
        return Err(HistoryError::NoHistory.into());
    }
    world.resource_scope(|world, mut history: bevy_ecs::world::Mut<History>| {
        history.end_group();
        let Some(mut step) = history.redo.pop() else {
            return Ok(false);
        };
        match step.apply(world) {
            Ok(()) => {
                history.undo.push(step);
                Ok(true)
            }
            Err(error) => {
                history.redo.push(step);
                Err(error)
            }
        }
    })
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::missing_panics_doc,
        reason = "a test stops at the first thing that is not as expected"
    )]

    use super::*;

    /// A count every command of a test adds one to, and what the commands may not do.
    #[derive(Resource, Default)]
    struct Count {
        /// The count.
        value: i32,
        /// The value the count may not be applied from.
        refuse_apply_at: Option<i32>,
        /// The value the count may not be reverted from.
        refuse_revert_at: Option<i32>,
    }

    /// Adds one to the [`Count`], unless the count refuses it.
    struct AddOne;

    impl ReversibleCommand for AddOne {
        fn apply(&mut self, world: &mut World) -> Result<(), BevyError> {
            let mut count = world.get_resource_mut::<Count>().ok_or("no count")?;
            if count.refuse_apply_at == Some(count.value) {
                return Err("refused".into());
            }
            count.value += 1;
            Ok(())
        }

        fn revert(&mut self, world: &mut World) -> Result<(), BevyError> {
            let mut count = world.get_resource_mut::<Count>().ok_or("no count")?;
            if count.refuse_revert_at == Some(count.value) {
                return Err("refused".into());
            }
            count.value -= 1;
            Ok(())
        }
    }

    /// A World with a history and a count.
    fn world() -> World {
        let mut world = World::new();
        world.init_resource::<History>();
        world.init_resource::<Count>();
        world
    }

    /// The count.
    fn count(world: &World) -> i32 {
        world.get_resource::<Count>().expect("a count").value
    }

    /// An abandoned group takes back what it applied and records nothing.
    #[test]
    fn an_abandoned_group_is_taken_back() {
        let mut world = world();
        world
            .get_resource_mut::<History>()
            .expect("a history")
            .begin_group();
        apply(&mut world, AddOne).expect("applied");
        apply(&mut world, AddOne).expect("applied");
        abandon_group(&mut world).expect("abandoned");

        assert_eq!(count(&world), 0);
        assert!(
            !world
                .get_resource::<History>()
                .expect("a history")
                .can_undo()
        );
    }

    /// A group that cannot be undone or redone whole is left as it was, and stays where it was in
    /// the history.
    #[test]
    fn a_group_moves_whole_or_not_at_all() {
        let mut world = world();
        world
            .get_resource_mut::<History>()
            .expect("a history")
            .begin_group();
        for _ in 0..3 {
            apply(&mut world, AddOne).expect("applied");
        }
        world
            .get_resource_mut::<History>()
            .expect("a history")
            .end_group();

        world
            .get_resource_mut::<Count>()
            .expect("a count")
            .refuse_revert_at = Some(1);
        assert!(undo(&mut world).is_err());
        assert_eq!(count(&world), 3, "the undo is taken back");

        world
            .get_resource_mut::<Count>()
            .expect("a count")
            .refuse_revert_at = None;
        assert!(undo(&mut world).expect("undone"));
        assert_eq!(count(&world), 0);

        world
            .get_resource_mut::<Count>()
            .expect("a count")
            .refuse_apply_at = Some(2);
        assert!(redo(&mut world).is_err());
        assert_eq!(count(&world), 0, "the redo is taken back");
        assert!(
            world
                .get_resource::<History>()
                .expect("a history")
                .can_redo()
        );
    }
}
