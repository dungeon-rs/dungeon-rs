# drs-shape-engine

The Engine that derives geometry from editable outlines.

Every operation reads one input, a [`Path`](crate::Path): a Wall's open line or
a Room's closed outline, its points with a part between each and the next,
straight or curved by one control point, and a closed outline's last part from
its last point back to its first. A Wall's parts are its segments and a Room's
its edges.

[`combine_outlines`](crate::combine_outlines) flattens an outline into its
line: chords never more than a thousandth of a cell off the curve, from the
first point to the last or round to the first again, each point tagged with
the part it lies on and the parameter along it. For a closed outline it also
fills what the line winds around, under the non-zero rule, as its floor. It is
given one outline at a time and combines it with none.
[`generate_walls`](crate::generate_walls) strokes that line at a thickness with
round joins at its points: an open line has round caps at its ends, and a
closed one joins round its first point and has no caps. The stroke is left out
along the stretches the Portals cover, ending squarely across the line at each
end of a stretch; an end of an open line a stretch reaches has no cap, and on
a closed line a stretch may run on past the first point.
[`split_wall`](crate::split_wall) splits a part at a parameter into two parts
of the shape it had, to single precision, a point added on a closed outline's
last edge becoming its last point.

[`anchor_portals`](crate::anchor_portals) says where each Portal set into an
outline stands: its centre at its part and parameter on the exact curve, the
angle of the line's direction there, or none where the part has no direction,
and the stretch of the line it covers, half its width either way along the
line, across the outline's points, stopping at an open line's ends and
wrapping round a closed one. [`anchor_portals_through`](crate::anchor_portals_through)
says where each anchor goes when a point is added or removed: kept on a new
part and parameter, so no Portal moves on the Level, or gone with the part of
the outline it sat in, the joined edge of a removed first point of a closed
outline being the last. Lengths along the line are measured over the flattened
line, summing its chords, so they too come out the same on every machine.

The floor is filled by `lyon_tessellation` over the same chords the Walls are
stroked along, so the floor's edge and the Walls' centre line agree; it is given
straight chords only, which it fills with arithmetic and comparisons.

Strokes are tessellated by the Engine itself, with no trigonometry: a round join
or cap is an arc subdivided by halving its angle until it is within the
tolerance, which takes only square roots, so the same outline gives the same
mesh, bit for bit, on every machine. The Engine is plain functions over its
input and the model's types; it defines no message and reads no file.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
