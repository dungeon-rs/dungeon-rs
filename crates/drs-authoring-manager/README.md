# drs-authoring-manager

The Manager that applies the Author's Commands to the Level: Place Element, Edit Element,
Remove Element, Set Portal into Wall, and Free Portal, each recorded in the history so that it
can be undone and redone.

A Place Element puts a Prop or a Portal of a chosen Asset, or a Wall through given points, on top
of its Layer; a Portal comes at its image's natural size, set into a Wall when the placement
anchors it and freestanding otherwise. An Edit Element moves an Element, and on a Wall also moves a point, bends or straightens
a segment, adds or removes a point, or sets the thickness or the colour; a drag sent as a gesture
is one step. Moving a point, bending, and the properties go through the history's generic field
command; adding and removing a point, the only edits that renumber a Wall's segments, are a step
of their own, and removing a point from a Wall of two points removes the Wall. An Edit Element also changes a Portal's width, a freestanding
Portal's position, rotation, and mirroring, and a set Portal's side and place along its Wall.
Set Portal into Wall anchors a Portal to a place along a Wall, and Free Portal clears the anchor,
leaving it where it stands. Adding or removing a Wall's point moves the anchors of the Portals
set into it in the same step, so none moves on the Level, and removes those whose part of the
Wall goes; removing a Wall removes its Portals with it, and either removal is answered with a
`PortalsRemoved` message naming them.

Once every Manager has handled the frame's Commands, Undo, and Redo, the Manager derives the shape
of every Wall whose points, segments, or thickness changed, or whose Portals changed, through the
shape Engine, leaving out the stretches its Portals cover, and sets the Element's box around its
points; a new colour keeps the shape. It moves each Portal set into such a Wall to where its
anchor puts it, turned to the Wall and mirrored when it faces the right, and sets every changed
Portal's size from its width and its image's proportions. A Portal whose anchor names no Wall of
its Level, or a segment its Wall lacks, stands where it was saved.

The Editor sends it `Apply`, `Undo`, and `Redo` messages; a Command that cannot be carried out is
answered with a `CommandFailed` message that says why, and an undo or redo that cannot be with a
`HistoryFailed`.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
