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

A [`PaintCache`](crate::PaintCache) holds one Terrain's coverage at 32 pixels per
cell in tiles of 512 pixels a side, keyed by their place in the Level's pixel
plane, negative places included, an absent tile being empty, and publishes them
as the model's coverage without copying a pixel; no one else looks inside it.
[`apply_stroke`](crate::apply_stroke) brings it up to the Terrain's strokes and
says whether any tile changed: appended strokes are composited onto the tiles
they touch, and when earlier strokes changed or went, only the tiles the
differing strokes touch are rasterized again from every stroke. Both are plain
functions over the model's types; the Manager that owns the cache keeps it.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
