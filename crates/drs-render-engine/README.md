# drs-render-engine

The Engine that draws the Level, in the editor and for Export.

The viewport systems of the [`RenderEnginePlugin`](crate::RenderEnginePlugin)
read the model through change detection and draw each Element as its kind's
descriptor in the Element kind registry says. They keep one sprite per Element
drawn as an image: a Prop's image loaded through the `lib://` asset source from
where the Project's resolution table says its Asset Reference loads on this
device, at the Element's size in cells, positioned at the Element's centre, and
stacked in the order of the Layer's children, each at its own depth. A Prop
whose image cannot be loaded or whose Asset is Missing, and an Element of a kind
this editor does not know, are drawn as the same flat coloured placeholder of
their recorded size.

An Element drawn as a stroked path, a Wall, is drawn as one mesh with a
flat-colour Material, built from the stroke mesh of its derived shape and
replaced whenever that shape changes, at its depth in the same stacking order as
the sprites. The Material blends, though the colour is opaque, so the mesh sorts
with the sprites by depth, and Walls of one colour share it. A Wall whose shape
has not been derived yet is not drawn that frame.

One Grid cell is one world unit, `x` to the right and `y` upwards, so an
Element's position and size in cells are its translation and size as drawn.
The projection is a 2D camera that follows the model's `Viewport`: it looks at
the cell the Viewport puts at the centre of its area, and shows a cell as
as many pixels as the Viewport's zoom says.

For the Export, `request_region` points one offscreen camera at a square of the
Level, in cells, and draws it into a texture of the tile size at a chosen number
of pixels per cell, through the same sprites, meshes, and depths as the viewport
over an opaque black background; `take_region` yields the pixels once the GPU
has handed them back, a few frames later. A region is captured only once every
image a sprite is loading has loaded or failed, and never in the frame the
camera was spawned in; a mesh has nothing to load. `release_regions` removes the
camera when the Export is done. Without a renderer, as in a headless editor
without Bevy's render plugins, the requests say so instead of drawing.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
