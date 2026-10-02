# drs-render-engine

The Engine that draws the Level, in the editor and for Export.

The viewport systems of the [`RenderEnginePlugin`](crate::RenderEnginePlugin)
read the model through change detection and draw each Element as its kind's
descriptor in the Element kind registry says. They keep one sprite per Element
drawn as an image: a Prop's image loaded through the `lib://` asset source from
where the Project's resolution table says its Asset Reference loads on this
device, at the Element's size in cells, positioned at the Element's centre, and
stacked in the order of the Layer's children, each at its own depth. A Portal
is drawn the same way, turned counter-clockwise by its rotation and flipped
across its length when it is mirrored; the turn is built through the
deterministic maths functions, so the Export is the same on every machine. A
Prop or a Portal whose image cannot be loaded or whose Asset is Missing, and an
Element of a kind this editor does not know, are drawn as the same flat coloured
placeholder of their recorded size, a Portal's turned and flipped as its image
would be.

An Element drawn as a stroked path, a Wall, is drawn as one mesh with a
flat-colour Material, built from the stroke mesh of its derived shape and
replaced whenever that shape changes, at its depth in the same stacking order as
the sprites; the derived stroke already leaves out the stretches the Wall's
Portals cover. The Material blends, though the colour is opaque, so the mesh sorts
with the sprites by depth, and Walls of one colour share it. A Wall whose shape
has not been derived yet is not drawn that frame.

An Element drawn as a painted surface, a Terrain, is drawn as one quad per
tile of its derived coverage, sixteen cells a side, with the masked tiled image
Material: its image repeated edge to edge across the Level at its natural size
from the Level's origin, as opaque at each point as the coverage there. The
Material's Shader is plain WGSL compiled into the Engine and added to the shader
assets at startup; it samples the image with the image's own sampler, so a Prop
and a Terrain share one loaded image. A tile's coverage is uploaded again when
its revision changes, and the quads sit at the Terrain's depth in the same
stacking order, blending so they sort with the sprites and meshes. While the
image loads, is Missing, or failed, the placeholder's flat colour is drawn masked
by the same coverage. A Terrain whose coverage has not been derived yet is not
drawn that frame.

An Element drawn as a filled outline, a Room, is drawn as two such meshes from
its derived shape: its floor in its floor colour, half a depth unit below its
Walls, which are stroked round its closed outline in its wall colour and
already leave out the stretches its Portals cover. Everything else before the
Room in the stacking order lies under both and everything after over both. A
Room whose shape has not been derived yet is not drawn that frame.

One Grid cell is one world unit, `x` to the right and `y` upwards, so an
Element's position and size in cells are its translation and size as drawn.
The projection is a 2D camera that follows the model's `Viewport`: it looks at
the cell the Viewport puts at the centre of its area, and shows a cell as
as many pixels as the Viewport's zoom says.

For the Export, `request_region` points one offscreen camera at a square of the
Level, in cells, and draws it into a texture of the tile size at a chosen number
of pixels per cell, through the same sprites, meshes, and depths as the viewport
over an opaque black background. The request carries each Terrain's coverage
computed over the region at its resolution, which the Engine draws for that
capture alone, one quad of the region's size per Terrain with the same Material,
in place of the viewport's coverage tiles, which the offscreen camera does not
see; the coverage's texels fall on the region's pixels one to one. The coverages
move into the request once it is accepted, uncopied, and stay with the caller
while it is refused. `take_region` yields the pixels once the GPU has handed
them back, a few frames later. A region is captured only once every image a
sprite or a Terrain is loading has loaded or failed and its Terrains' coverages
are in place, and never in the frame the camera was spawned in; a mesh has
nothing to load. `release_regions` removes the camera when the Export is done.
Without a renderer, as in a headless editor without Bevy's render plugins, the
requests say so instead of drawing.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
