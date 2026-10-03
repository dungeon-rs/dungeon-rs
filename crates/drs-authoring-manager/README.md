# drs-authoring-manager

The Manager that applies the Author's Commands to the Level: Place Element, Edit Element,
Remove Element, Set Portal into Wall, Free Portal, and Paint, each recorded in the history so
that it can be undone and redone.

A Place Element puts a Prop or a Portal of a chosen Asset, or a Wall or a Room through given
points, on top of its Layer; a Portal comes at its image's natural size, set into a Wall or a
Room when the placement anchors it and freestanding otherwise. An Edit Element moves an Element, and on a Wall also moves
a point, bends or straightens a segment, adds or removes a point, or sets the thickness or the
colour; a drag sent as a gesture is one step. Moving a point, bending, and the properties go
through the history's generic field command; adding and removing a point, the only edits that
renumber a Wall's segments, are a step of their own, and removing a point from a Wall of two
points removes the Wall. A Room is edited as a Wall is, its edges in place of the segments, the
edge from its last point back to its first included, and its floor colour besides; removing a
point joins the two edges at it into one straight edge, and removing a point from a Room of three
points removes the Room. An Edit Element also changes a Portal's width, a freestanding Portal's
position, rotation, and mirroring, and a set Portal's side and place along its Wall. Set Portal
into Wall anchors a Portal to a place along a Wall or a Room's Walls, and Free Portal clears the
anchor, leaving it where it stands. Adding or removing a point of a Wall or a Room moves the
anchors of the Portals set into it in the same step, so none moves on the Level, and removes
those whose part of it goes; removing a Wall or a Room removes its Portals with it, and either
removal is answered with a `PortalsRemoved` message naming them. A Command recorded as several
steps that fails halfway is taken back whole.

Once every Manager has handled the frame's Commands, Undo, and Redo, the Manager derives the shape
of every Wall and every Room whose points, segments or edges, or thickness changed, or whose
Portals changed, through the shape Engine, a Room's floor with its Walls, leaving out the
stretches its Portals cover, and sets the Element's box around its points; a new colour keeps
the shape. It moves each Portal set into such a Wall or Room to where its anchor puts it, turned
to the line and mirrored when it faces the right, and sets every changed Portal's size from its
width and its image's proportions. A Portal whose anchor names no Wall or Room of its Level, or a
part its host lacks, stands where it was saved and is moved, turned, and mirrored as a
freestanding one, and a point added to its host moves its anchor past the new part, so it goes
on standing there.

A Paint lays one stroke on the topmost Terrain of its Layer as one step that undo takes off the end
again; on a Layer with no Terrain it places one of the chosen Asset's image under every Element on
the Layer, holding the stroke, in the same step. A Paint naming no Asset paints with the Terrain's
own image, and one naming another image than the Terrain's is refused. A Paint whose stroke erases
is laid on the topmost Terrain whatever image it names, and refused on a Layer with no Terrain. An
Edit Element of a Terrain changes its image, every stroke kept, or one of its strokes, named by its
number: a point of its path, its whole path moved to a new centre, its Brush settings, or whether
it erases, each through the generic field command so a drag is one step; naming the image or the
erasing it already has records nothing. Removing a stroke, the only edit that renumbers strokes, is
a step of its own that puts it back at its number on undo, and removing the only stroke removes the
Terrain. After every Manager has handled the frame's Commands, Undo, and Redo, the Manager brings
each changed Terrain's tiled coverage up to its strokes through the paint Engine, keeping the
Engine's cache, which only the Engine looks inside, in a component of its own; it publishes the
coverage again when a tile changed, and sets the Element's box around the strokes.

The Editor sends it `Apply`, `Undo`, and `Redo` messages; a Command that cannot be carried out is
answered with a `CommandFailed` message that says why, and an undo or redo that cannot be with a
`HistoryFailed`.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
