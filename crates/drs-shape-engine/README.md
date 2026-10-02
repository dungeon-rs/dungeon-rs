# drs-shape-engine

The Engine that derives geometry from editable outlines.

For a drawn Wall, [`generate_walls`](crate::generate_walls) derives the shape it
is drawn and picked by: its line flattened into chords, never more than a
thousandth of a cell off the curve, each point tagged with the segment it lies
on and the parameter along it, and the stroke mesh at the Wall's thickness with
round joins at its points and round caps at its ends.
[`split_wall`](crate::split_wall) splits a segment at a parameter into two
segments of the shape it had, to single precision.

Strokes are tessellated by the Engine itself, with no trigonometry: a round join
or cap is an arc subdivided by halving its angle until it is within the
tolerance, which takes only square roots, so the same Wall gives the same mesh,
bit for bit, on every machine. The Engine is plain functions over the model's
types; it defines no message and reads no file.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
