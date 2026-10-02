# Guidelines

- [Atomic file replacement](./atomic-file-replacement.md): use when a ResourceAccess writes a file the Author must never find half-written.
- [Development switch](./development-switch.md): use when adding a switch the editor needs only for development or autonomous verification.
- [Editor modal dialog](./editor-modal-dialog.md): use when the Editor asks or tells the Author something that must be answered before the window is used again.
- [Failure shown in the status line](./status-line-failure.md): use when the Editor learns of a failure the Author must see without a modal.
- [Headless seam test](./headless-seam-test.md): use when writing a test at a Manager seam.
- [Message handler of a Manager](./manager-message-handler.md): use when a Manager handles a new Command or request message.
- [Reversible command](./reversible-command.md): use when a Manager records a new undoable step.
- [Serialisable component](./serialisable-component.md): use when a crate adds a component a Project file must hold.
