# Guidelines

- [Atomic file replacement](./atomic-file-replacement.md): use when a ResourceAccess writes a file the Author must never find half-written.
- [Background worker pool in a ResourceAccess](./background-worker-pool.md): use when a ResourceAccess runs a queue of jobs a Manager feeds, on threads of its own.
- [Derived model component](./derived-model-component.md): use when a Client or an Engine needs a value computed from the model by an Engine it may not depend on.
- [Development switch](./development-switch.md): use when adding a switch the editor needs only for development or autonomous verification.
- [Edit that carries anchored Elements](./edit-carrying-anchored-elements.md): use when an authoring Command renumbers or removes what other Elements are anchored to.
- [Editor modal dialog](./editor-modal-dialog.md): use when the Editor asks or tells the Author something that must be answered before the window is used again.
- [Element kind](./element-kind.md): use when the editor gains a new kind of Element.
- [Engine cache held by a Manager](./engine-cache-held-by-a-manager.md): use when an Engine keeps work between calls whose result others read.
- [Failure shown in the status line](./status-line-failure.md): use when the Editor learns of a failure the Author must see without a modal.
- [Headless seam test](./headless-seam-test.md): use when writing a test at a Manager seam.
- [Message handler of a Manager](./manager-message-handler.md): use when a Manager handles a new Command or request message.
- [Reversible command](./reversible-command.md): use when a Manager records a new undoable step.
- [Serialisable component](./serialisable-component.md): use when a crate adds a component a Project file must hold.
- [Timed budget test](./timed-budget-test.md): use when a test bounds how long the code takes.
- [Tool-strip option as a gesture](./tool-strip-option-gesture.md): use when the tool strip shows a property of the selected Element in a widget the Author holds while it changes.
