# drs-shape-engine

The Engine that derives geometry from editable outlines.

For a drawn Wall, [`generate_walls`](crate::generate_walls) derives the shape it
is drawn and picked by: its line flattened into chords, never more than a
thousandth of a cell off the curve, each point tagged with the segment it lies
on and the parameter along it, and the stroke mesh at the Wall's thickness with
round joins at its points and round caps at its ends, left out along the
stretches its Portals cover: the stroke ends squarely across the line at each
end of a stretch, and an end of the Wall a stretch reaches has no cap.
[`split_wall`](crate::split_wall) splits a segment at a parameter into two
segments of the shape it had, to single precision.

[`anchor_portals`](crate::anchor_portals) says where each Portal set into a
Wall stands: its centre at its segment and parameter on the exact curve, the
angle of the line's direction there, or none where the segment has no
direction, and the stretch of the line it covers, half its width either way
along the line, across the Wall's points, and stopping at its ends.
[`anchor_portals_through`](crate::anchor_portals_through) says where each anchor
goes when a point is added or removed: kept on a new segment and parameter, so
no Portal moves on the Level, or gone with the part of the Wall it sat in.
Lengths along the line are measured over the flattened line, summing its
chords, so they too come out the same on every machine.

Strokes are tessellated by the Engine itself, with no trigonometry: a round join
or cap is an arc subdivided by halving its angle until it is within the
tolerance, which takes only square roots, so the same Wall gives the same mesh,
bit for bit, on every machine. The Engine is plain functions over the model's
types; it defines no message and reads no file.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
