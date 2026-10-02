# drs-output-access

The `ResourceAccess` that writes exported images.

An Export is written in parts, because a Level exported at a high resolution
is far larger than should sit in memory at once: `begin_image` opens a PNG of
the final size, `write_tile` takes the rendered tiles in row-major order, and
`finish_image` closes the file. The tiles of one row are assembled into a band
as high as a tile and as wide as the image, and each completed band is
streamed into the encoder, so an image of twenty thousand pixels a side costs
one band of memory, not the whole image. A tile that reaches past the right or
bottom edge is clipped to the image.

The PNG is 8-bit RGBA with fixed compression and filter settings, so the same
tiles always produce the same bytes. Until `finish_image` returns, the image is
written to a temporary `.part` file beside the chosen path, flushed to the disk,
and renamed over the path at the end; a failure before that, or dropping the
writer without finishing, removes the temporary file and leaves whatever was at
the path untouched.

This crate uses no Bevy.

## Features

- `default`: nothing is enabled by default.
- `dev`: debug tooling for development.
