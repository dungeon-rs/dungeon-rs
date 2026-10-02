# drs-authoring-manager

The Manager that applies the Author's Commands to the Level: Place Element, Edit Element, and
Remove Element, each recorded in the history so that it can be undone and redone.

A Place Element puts a Prop of a chosen Asset or a Wall through given points on top of its
Layer. An Edit Element moves an Element, and on a Wall also moves a point, bends or straightens
a segment, adds or removes a point, or sets the thickness or the colour; a drag sent as a gesture
is one step. Moving a point, bending, and the properties go through the history's generic field
command; adding and removing a point, the only edits that renumber a Wall's segments, are a step
of their own, and removing a point from a Wall of two points removes the Wall. Once every Manager
has handled the frame's Commands, Undo, and Redo, the Manager derives the shape of every Wall
whose points, segments, or thickness changed through the shape Engine and sets the Element's box
around its points; a new colour keeps the shape.

The Editor sends it `Apply`, `Undo`, and `Redo` messages; a Command that cannot be carried out is
answered with a `CommandFailed` message that says why, and an undo or redo that cannot be with a
`HistoryFailed`.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
