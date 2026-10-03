# drs-paint-engine

The Engine that turns painted strokes into coverage: how much of a painted
Element's Material shows where.

[`rasterize`](crate::rasterize) computes the coverage of strokes over any region
of the Level at any number of pixels per cell, one byte a pixel, rows from the
top, on the CPU. A pixel's coverage is taken at its centre, found from its
whole-pixel index in the Level's pixel plane, so it is the same in every region
that holds it, and only square roots and arithmetic go into it, so it is the same
on every machine. A stroke's coverage is its strength within the hardness of its
radius of its path and falls off smoothly to nothing at the radius, the same
however many of its segments pass near a point. Strokes composite in order onto
nothing: a stroke that paints raises the coverage to its own where that is
higher, and an erase lowers it to one minus its own where that is lower, so an
erase is exact in one pass and takes only from the strokes laid before it.

A [`PaintCache`](crate::PaintCache), made for the GPU or the CPU once and for good
by [`PaintCache::new`](crate::PaintCache::new), holds one Terrain's coverage in
tiles of 512 pixels a side, keyed by their place in the Level's pixel plane at
their band, negative places included, an absent tile being empty, and publishes
them as the model's coverage without copying a pixel; no one else looks inside it.
[`apply_stroke`](crate::apply_stroke) brings it up to the Terrain's strokes and to
the view the Viewport shows, and says whether anything published changed. It
compares the strokes it holds with the Terrain's from both ends: appended strokes,
erases included, are drawn onto the tiles they reach, and otherwise only the tiles
the strokes between the shared ends reach, as they were and as they are, are
rasterized again from every stroke, so editing or removing one stroke recomputes
that stroke's tiles alone. Both are plain functions over the model's types; the
Manager that owns the cache keeps it.

Where the editor renders, the tiles are rasterized on the GPU through
[`StrokeRasterizer`](crate::StrokeRasterizer), the system parameter the Manager's
system takes: the cache holds a base band of 32 pixels per cell wherever the
strokes reach and, while the zoom calls for a closer band (64, 128, or 256, the
largest not above the zoom, kept from 0.9 to 2.2 times it), that band's tiles
that meet the view and that some stroke reaches, letting them go once they lie
more than a tile's width from the view. Each tile is an R8 image on the GPU
alone, which the model names by an opaque identity. The work is handed to
[`PaintEnginePlugin`](crate::PaintEnginePlugin), which in the render world draws
one pass per tile, clearing it or drawing onto it, with one quad per segment of
each stroke and a maximum blend for paint and a minimum blend for erase, before
any camera renders the frame; work that finds the pipeline not compiled yet waits
for a later frame, a pass that clears a tile replacing the work still waiting for
it, and is let go, with a warning, should the pipeline fail to compile. Without a
renderer, or without the stroke Shader, the cache holds the base band alone,
rasterized on the CPU, whatever the zoom.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
