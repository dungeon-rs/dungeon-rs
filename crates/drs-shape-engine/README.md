# drs-shape-engine

The Engine that derives geometry from editable outlines.

Every operation reads outlines as [`Path`](crate::Path)s: a Wall's open line or
a Room's closed outline, its points with a part between each and the next,
straight or curved by one control point, and a closed outline's last part from
its last point back to its first. A Wall's parts are its segments and a Room's
its edges.

[`combine_outlines`](crate::combine_outlines) takes the
[`Outline`](crate::Outline)s of a Layer in stacking order, each a path and
whether it cuts, and derives for each a
[`CombinedOutline`](crate::CombinedOutline): its whole line flattened into
chords never more than a thousandth of a cell off the curve, each point tagged
with the part it lies on and the parameter along it; its floor; the Walls drawn
in its look; the places of its parts where any Wall runs; and the last outline
of the combination it belongs to. The closed outlines are folded in stacking
order through `i_overlay`'s `EdgeOverlay`, behind one integer adapter for every
pass so coincident edges stay coincident: each run of outlines that do not cut
is a union onto the floor so far, each run that cuts a difference from it.
Every edge carries the part and the parameters it is a piece of through every
split and merge, so a Wall runs along each piece of an edge with the combined
floor on one side only, drawn in the look of the last outline whose edge lies
there, and along every stretch two outlines that do not cut share from opposite
sides, kept from the merges' records, with any third edge along it found
again, and clipped to the combined floor where a later cut reaches it. Each
outline's Walls are rebuilt on its exact curves, their ends on the vertex where
another outline's Wall begins. An outline that does not cut keeps its own floor,
filled under the non-zero rule, less what later outlines that cut take away;
one that cuts has none. Outlines whose floors overlap or touch, through others
and cuts included, form one combination. An open line combines with nothing:
its Wall is its whole line.
[`generate_walls`](crate::generate_walls) strokes an outline's Walls at a
thickness with round joins at their points: a Wall that runs round joins round
its first point and has no ends, and any other has round caps. The stroke is
left out along the stretches the Portals cover, ending squarely across the line
at each end of a stretch; an end a stretch reaches has no cap, and on a closed
outline a stretch may run on past the first point.
[`split_wall`](crate::split_wall) splits a part at a parameter into two parts
of the shape it had, to single precision, a point added on a closed outline's
last edge becoming its last point.

[`anchor_portals`](crate::anchor_portals) says where each Portal set into an
outline of a combination stands: its centre at its part and parameter on its
own outline's exact curve, the angle of the line's direction there, or none
where the part has no direction, and, where a Wall runs at its centre, the
stretches of the Walls it covers, half its width either way along the Wall it
stands in: round the edge of the combined floor and onto another outline's
Wall, wrapping round a closed one, or along a drawn Wall or a Wall two Rooms
share, stopping at its ends. [`anchor_portals_through`](crate::anchor_portals_through)
says where each anchor goes when a point is added or removed: kept on a new
part and parameter, so no Portal moves on the Level, or gone with the part of
the outline it sat in, the joined edge of a removed first point of a closed
outline being the last. Lengths along the line are measured over the flattened
line, summing its chords, so they too come out the same on every machine.

The floor is filled by `lyon_tessellation` over the same chords the Walls are
stroked along, so the floor's edge and the Walls' centre line agree; it is given
straight chords only, which it fills with arithmetic and comparisons.

[`snap`](crate::snap) says where a point being placed or dragged goes: to the
nearest of the given points of the Walls and Rooms of its Level within its
reach, the later in the stacking order of two as near, leaving out the point
being dragged, and taking that point's coordinates exactly; with none within
reach, to the nearest Grid corner, each coordinate rounded to a whole number of
cells, halfway away from zero. A whole Wall or Room being dragged moves by the
pointer's travel rounded the same way. It compares squared distances and
rounds, so it too answers the same on every machine.

Strokes are tessellated by the Engine itself, with no trigonometry: a round join
or cap is an arc subdivided by halving its angle until it is within the
tolerance, which takes only square roots, so the same outline gives the same
mesh, bit for bit, on every machine. The Engine is plain functions over its
input and the model's types; it defines no message and reads no file.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
