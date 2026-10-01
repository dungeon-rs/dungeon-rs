# drs-render-engine

The Engine that draws the Level, in the editor and for Export.

The viewport systems of the [`RenderEnginePlugin`](crate::RenderEnginePlugin)
read the model through change detection and keep one sprite per Prop: its image
loaded through the `lib://` asset source at the Element's size in cells,
positioned at the Element's centre, and stacked in the order of the Layer's
children, each at its own depth. A Prop whose image cannot be loaded is drawn
as a flat coloured placeholder of its recorded size.

One Grid cell is one world unit, `x` to the right and `y` upwards, so an
Element's position and size in cells are its translation and size as drawn.
The projection is a 2D camera that follows the model's `Viewport`: it looks at
the cell the Viewport puts at the centre of its area, and shows a cell as as
many pixels as the Viewport's zoom says.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
