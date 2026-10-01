# drs-history

The Utility that records Commands for undo and redo.

A domain-agnostic stack of reversible commands over the Bevy `World`. A
[`ReversibleCommand`](crate::ReversibleCommand) knows how to apply itself and how
to take itself back; [`apply`](crate::apply) carries one out and records it,
[`undo`](crate::undo) and [`redo`](crate::redo) walk the stack. Several commands
recorded while a group is open form one step, so a gesture such as a drag is
undone as a whole; [`apply_step`](crate::apply_step) closes an open group first,
for a command that is a step of its own. Recording a new step discards the
steps that were undone. [`History::position`](crate::History::position) tells
where the history stands, so whoever saved the World can later tell whether a
step has been recorded, or undone, since; [`History::clear`](crate::History::clear)
forgets every step.

Two generic commands cover most needs without a command per property:
[`SetField`](crate::SetField) swaps one value by reflect path, and
[`Snapshot`](crate::Snapshot) removes an entity with every reflected component
and restores it on undo. Both name their entity through a
[`Target`](crate::Target), so a stable identity rather than an entity handle can
key them.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
