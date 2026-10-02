#![doc = include_str!("../README.md")]

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use tempfile::TempPath;

/// Bytes per RGBA8 pixel.
const BYTES_PER_PIXEL: usize = 4;

/// How many rows of the image the encoder buffers before compressing them.
const ROWS_PER_CHUNK: usize = 64;

/// What can go wrong while writing an image.
#[derive(Debug, thiserror::Error)]
pub enum OutputError {
    /// The image has no pixels along one side.
    #[error("an image of {width} by {height} pixels cannot be written")]
    EmptyImage {
        /// The width that was asked for.
        width: u32,
        /// The height that was asked for.
        height: u32,
    },
    /// A file could not be created, written, renamed, or removed.
    #[error("cannot {action} {}: {source}", path.display())]
    Io {
        /// What was attempted, such as `create` or `write`.
        action: &'static str,
        /// The file.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },
    /// The encoder refused the image or its data.
    #[error("the image {} cannot be encoded: {reason}", path.display())]
    Encoding {
        /// The file.
        path: PathBuf,
        /// What the encoder said.
        reason: String,
    },
    /// A tile arrived somewhere else than where the next tile belongs.
    #[error(
        "the tile at ({x}, {y}) is out of order; the next tile belongs at ({expected_x}, {expected_y})"
    )]
    OutOfOrder {
        /// The column, in pixels, the tile claimed.
        x: u32,
        /// The row, in pixels, the tile claimed.
        y: u32,
        /// The column the next tile belongs at.
        expected_x: u32,
        /// The row the next tile belongs at.
        expected_y: u32,
    },
    /// A tile is not as high as the band it joins, or has no pixels.
    #[error("the tile at ({x}, {y}) is {width} by {height} pixels; the band is {band} rows high")]
    UnevenTile {
        /// The column of the tile.
        x: u32,
        /// The row of the tile.
        y: u32,
        /// The tile's width.
        width: u32,
        /// The tile's height.
        height: u32,
        /// The height of the band the tile joins.
        band: u32,
    },
    /// A tile's data does not match its size.
    #[error("the tile at ({x}, {y}) holds {actual} bytes; {expected} were expected")]
    WrongSize {
        /// The column of the tile.
        x: u32,
        /// The row of the tile.
        y: u32,
        /// How many bytes its size needs.
        expected: usize,
        /// How many bytes it holds.
        actual: usize,
    },
    /// `finish_image` was called before the last row was written.
    #[error("the image is incomplete: {written} of {height} rows were written")]
    Incomplete {
        /// How many rows were written.
        written: u32,
        /// How many rows the image has.
        height: u32,
    },
}

/// One rendered tile of the image: its position and size in pixels, counted from the top-left
/// corner, and its RGBA8 pixels, rows top to bottom.
///
/// A tile may reach past the right or bottom edge of the image; the part outside is dropped.
#[derive(Debug, Clone, Copy)]
pub struct Tile<'a> {
    /// The column of the tile's left edge.
    pub x: u32,
    /// The row of the tile's top edge.
    pub y: u32,
    /// The tile's width in pixels.
    pub width: u32,
    /// The tile's height in pixels.
    pub height: u32,
    /// `width * height * 4` bytes of RGBA8 pixels.
    pub rgba: &'a [u8],
}

/// An image being written: a PNG streamed band by band to a temporary file beside its path.
///
/// Dropping the writer without [`finish_image`] removes the temporary file.
pub struct ImageWriter {
    /// Where the image ends up.
    path: PathBuf,
    /// Where the image is written until it is complete; removed when dropped, `None` once the
    /// file has taken the path's place or been removed.
    partial: Option<TempPath>,
    /// The encoder, `None` once finished or failed.
    encoder: Option<png::StreamWriter<'static, BufWriter<File>>>,
    /// The image's width in pixels.
    width: u32,
    /// The image's height in pixels.
    height: u32,
    /// The row of the band being assembled.
    band_top: u32,
    /// The height of the band being assembled, decided by its first tile; `0` before it.
    band_height: u32,
    /// The column the next tile of the band belongs at.
    band_x: u32,
    /// The band's pixels, `width * band_height * 4` bytes.
    band: Vec<u8>,
}

/// `BeginImage`: starts a PNG of `width` by `height` pixels at `path`.
///
/// The file is created beside `path` under a temporary name ending in `.part` and takes the
/// path's place when [`finish_image`] completes.
///
/// # Errors
///
/// [`OutputError::EmptyImage`] when a side is zero, [`OutputError::Io`] when the file cannot be
/// created, or [`OutputError::Encoding`] when the encoder refuses the size.
pub fn begin_image(path: &Path, width: u32, height: u32) -> Result<ImageWriter, OutputError> {
    if width == 0 || height == 0 {
        return Err(OutputError::EmptyImage { width, height });
    }
    let (file, partial) = partial_file(path)
        .map_err(|source| OutputError::Io {
            action: "create a file beside",
            path: path.to_path_buf(),
            source,
        })?
        .into_parts();
    let mut encoder = png::Encoder::new(BufWriter::new(file), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    // Fixed settings keep the bytes the same for the same pixels, run after run.
    encoder.set_compression(png::Compression::Fast);
    encoder.set_filter(png::Filter::Adaptive);
    encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    let row_bytes = width as usize * BYTES_PER_PIXEL;
    let encoder = encoder
        .write_header()
        .and_then(|writer| writer.into_stream_writer_with_size(row_bytes * ROWS_PER_CHUNK));
    // A failure here drops `partial`, which removes the file.
    let encoder = encoder.map_err(|error| OutputError::Encoding {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })?;
    Ok(ImageWriter {
        path: path.to_path_buf(),
        partial: Some(partial),
        encoder: Some(encoder),
        width,
        height,
        band_top: 0,
        band_height: 0,
        band_x: 0,
        band: Vec::new(),
    })
}

/// `WriteTile`: adds the next tile, in row-major order from the top-left corner.
///
/// Tiles of one row must be as high as each other; the row's band is streamed into the file
/// once its last tile, the one reaching the right edge, has arrived.
///
/// # Errors
///
/// [`OutputError::OutOfOrder`], [`OutputError::UnevenTile`], or [`OutputError::WrongSize`] when
/// the tile does not fit where the next tile belongs, or [`OutputError::Encoding`] when the band
/// could not be written. After an error the writer is spent: finishing it fails and the partial
/// file is removed.
pub fn write_tile(writer: &mut ImageWriter, tile: Tile<'_>) -> Result<(), OutputError> {
    let expected = (writer.band_x, writer.band_top);
    if (tile.x, tile.y) != expected {
        return Err(OutputError::OutOfOrder {
            x: tile.x,
            y: tile.y,
            expected_x: expected.0,
            expected_y: expected.1,
        });
    }
    if writer.band_height == 0 {
        if tile.height == 0 || tile.width == 0 {
            return Err(OutputError::UnevenTile {
                x: tile.x,
                y: tile.y,
                width: tile.width,
                height: tile.height,
                band: 0,
            });
        }
        writer.band_height = tile.height.min(writer.height - writer.band_top);
        writer.band =
            vec![0; writer.width as usize * writer.band_height as usize * BYTES_PER_PIXEL];
    } else if tile.height.min(writer.height - writer.band_top) != writer.band_height
        || tile.width == 0
    {
        return Err(OutputError::UnevenTile {
            x: tile.x,
            y: tile.y,
            width: tile.width,
            height: tile.height,
            band: writer.band_height,
        });
    }
    let expected_bytes = tile.width as usize * tile.height as usize * BYTES_PER_PIXEL;
    if tile.rgba.len() != expected_bytes {
        return Err(OutputError::WrongSize {
            x: tile.x,
            y: tile.y,
            expected: expected_bytes,
            actual: tile.rgba.len(),
        });
    }

    // Copy the part of the tile inside the image into the band, row by row.
    let inside_width = tile.width.min(writer.width - tile.x) as usize;
    let tile_row = tile.width as usize * BYTES_PER_PIXEL;
    let band_row = writer.width as usize * BYTES_PER_PIXEL;
    let x_offset = tile.x as usize * BYTES_PER_PIXEL;
    for row in 0..writer.band_height as usize {
        let source = &tile.rgba[row * tile_row..row * tile_row + inside_width * BYTES_PER_PIXEL];
        let start = row * band_row + x_offset;
        writer.band[start..start + source.len()].copy_from_slice(source);
    }

    writer.band_x = tile.x.saturating_add(tile.width);
    if writer.band_x >= writer.width {
        let band = std::mem::take(&mut writer.band);
        let Some(encoder) = writer.encoder.as_mut() else {
            return Err(OutputError::Encoding {
                path: writer.path.clone(),
                reason: "the writer is spent".to_owned(),
            });
        };
        if let Err(error) = encoder.write_all(&band) {
            writer.spend();
            return Err(OutputError::Encoding {
                path: writer.path.clone(),
                reason: error.to_string(),
            });
        }
        writer.band_top += writer.band_height;
        writer.band_height = 0;
        writer.band_x = 0;
    }
    Ok(())
}

/// `FinishImage`: closes the PNG, flushes it to the disk, and moves it to its path.
///
/// # Errors
///
/// [`OutputError::Incomplete`] when rows are still missing, [`OutputError::Encoding`] when the
/// encoder could not close the file, or [`OutputError::Io`] when the finished file could not be
/// flushed or take the path's place. In every case the partial file is removed.
pub fn finish_image(mut writer: ImageWriter) -> Result<(), OutputError> {
    let Some(encoder) = writer.encoder.take() else {
        writer.spend();
        return Err(OutputError::Encoding {
            path: writer.path.clone(),
            reason: "the writer is spent".to_owned(),
        });
    };
    if writer.band_top != writer.height || writer.band_height != 0 {
        writer.spend();
        return Err(OutputError::Incomplete {
            written: writer.band_top,
            height: writer.height,
        });
    }
    if let Err(error) = encoder.finish() {
        writer.spend();
        return Err(OutputError::Encoding {
            path: writer.path.clone(),
            reason: error.to_string(),
        });
    }
    let Some(partial) = writer.partial.take() else {
        return Err(OutputError::Encoding {
            path: writer.path.clone(),
            reason: "the writer is spent".to_owned(),
        });
    };
    // The encoder owned the file and closed it; the bytes are flushed to the disk through the
    // path before the file takes the target's place, so a crash right after leaves a whole image.
    // The file is opened for writing because flushing needs write access on Windows.
    let io = |action: &'static str, source: std::io::Error| OutputError::Io {
        action,
        path: writer.path.clone(),
        source,
    };
    File::options()
        .write(true)
        .open(&partial)
        .and_then(|file| file.sync_all())
        .map_err(|source| io("flush", source))?;
    partial
        .persist(&writer.path)
        .map_err(|error| io("rename", error.error))
}

impl ImageWriter {
    /// Drops the encoder and removes the partial file, if any.
    fn spend(&mut self) {
        self.encoder = None;
        self.partial = None;
    }
}

impl Drop for ImageWriter {
    fn drop(&mut self) {
        self.spend();
    }
}

impl std::fmt::Debug for ImageWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageWriter")
            .field("path", &self.path)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("rows_written", &self.band_top)
            .finish_non_exhaustive()
    }
}

/// The temporary file an image is written to until it is complete: the path's file name, a
/// random infix, and a `.part` suffix, in the same directory so the final rename never crosses a
/// file system.
///
/// The file is created as readable as any file the Author makes: a temporary file is private by
/// default, but an Export is for sharing, so it is created with the usual mode, which the
/// process's umask narrows as it does for every new file.
///
/// # Errors
///
/// The error of creating the file.
fn partial_file(path: &Path) -> std::io::Result<tempfile::NamedTempFile> {
    let name = path.file_name().map_or_else(
        || std::ffi::OsString::from("image"),
        std::ffi::OsStr::to_os_string,
    );
    let beside = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut prefix = name;
    prefix.push(".");
    let mut builder = tempfile::Builder::new();
    builder.prefix(&prefix).suffix(".part");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        builder.permissions(std::fs::Permissions::from_mode(0o666));
    }
    builder.tempfile_in(beside)
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::missing_panics_doc,
        reason = "a test stops at the first thing that is not as expected"
    )]
    use super::*;
    use std::fs;

    /// The files in `dir`, by name.
    fn files_in(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    /// A tile filled with one colour.
    fn solid(width: u32, height: u32, colour: [u8; 4]) -> Vec<u8> {
        colour.repeat(width as usize * height as usize)
    }

    /// Decodes a PNG into its size and RGBA8 bytes.
    fn decode(path: &Path) -> (u32, u32, Vec<u8>) {
        let decoder = png::Decoder::new(std::io::BufReader::new(File::open(path).unwrap()));
        let mut reader = decoder.read_info().unwrap();
        let mut buffer = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut buffer).unwrap();
        buffer.truncate(info.buffer_size());
        (info.width, info.height, buffer)
    }

    #[test]
    fn tiles_overhanging_the_edges_are_clipped() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("image.png");
        let mut writer = begin_image(&path, 5, 3).unwrap();
        let red = [255, 0, 0, 255];
        let blue = [0, 0, 255, 255];
        for (x, y, colour) in [(0, 0, red), (4, 0, blue), (0, 2, blue), (4, 2, red)] {
            let rgba = solid(4, 2, colour);
            write_tile(
                &mut writer,
                Tile {
                    x,
                    y,
                    width: 4,
                    height: 2,
                    rgba: &rgba,
                },
            )
            .unwrap();
        }
        finish_image(writer).unwrap();

        let (width, height, pixels) = decode(&path);
        assert_eq!((width, height), (5, 3));
        let pixel = |x: usize, y: usize| &pixels[(y * 5 + x) * 4..(y * 5 + x) * 4 + 4];
        assert_eq!(pixel(0, 0), red);
        assert_eq!(pixel(3, 1), red);
        assert_eq!(pixel(4, 0), blue);
        assert_eq!(pixel(0, 2), blue);
        assert_eq!(pixel(4, 2), red);
        assert_eq!(files_in(dir.path()), vec!["image.png"]);
    }

    #[test]
    fn a_dropped_writer_leaves_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("image.png");
        let writer = begin_image(&path, 4, 4).unwrap();
        let partial = files_in(dir.path());
        assert_eq!(partial.len(), 1);
        assert!(partial[0].starts_with("image.png."));
        assert_eq!(Path::new(&partial[0]).extension(), Some("part".as_ref()));
        drop(writer);
        assert!(!path.exists());
        assert!(files_in(dir.path()).is_empty());
    }

    #[test]
    fn an_incomplete_image_is_refused_and_removed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("image.png");
        let mut writer = begin_image(&path, 2, 4).unwrap();
        let rgba = solid(2, 2, [1, 2, 3, 255]);
        write_tile(
            &mut writer,
            Tile {
                x: 0,
                y: 0,
                width: 2,
                height: 2,
                rgba: &rgba,
            },
        )
        .unwrap();
        assert!(matches!(
            finish_image(writer),
            Err(OutputError::Incomplete {
                written: 2,
                height: 4
            })
        ));
        assert!(!path.exists());
        assert!(files_in(dir.path()).is_empty());
    }

    #[test]
    fn a_tile_out_of_order_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let mut writer = begin_image(&dir.path().join("image.png"), 4, 4).unwrap();
        let rgba = solid(2, 2, [0; 4]);
        let tile = Tile {
            x: 2,
            y: 0,
            width: 2,
            height: 2,
            rgba: &rgba,
        };
        assert!(matches!(
            write_tile(&mut writer, tile),
            Err(OutputError::OutOfOrder {
                expected_x: 0,
                expected_y: 0,
                ..
            })
        ));
    }

    /// The finished image is as readable as any file made in the same place.
    #[cfg(unix)]
    #[test]
    fn the_image_is_as_readable_as_any_file() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("image.png");
        let mut writer = begin_image(&path, 1, 1).unwrap();
        let rgba = solid(1, 1, [0; 4]);
        write_tile(
            &mut writer,
            Tile {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
                rgba: &rgba,
            },
        )
        .unwrap();
        finish_image(writer).unwrap();
        let plain = dir.path().join("plain");
        fs::write(&plain, b"").unwrap();

        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode(),
            fs::metadata(&plain).unwrap().permissions().mode()
        );
    }

    #[test]
    fn the_same_tiles_give_the_same_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let mut files = Vec::new();
        for name in ["a.png", "b.png"] {
            let path = dir.path().join(name);
            let mut writer = begin_image(&path, 3, 2).unwrap();
            let rgba: Vec<u8> = (0..3u8 * 2 * 4).map(|i| i.wrapping_mul(37)).collect();
            write_tile(
                &mut writer,
                Tile {
                    x: 0,
                    y: 0,
                    width: 3,
                    height: 2,
                    rgba: &rgba,
                },
            )
            .unwrap();
            finish_image(writer).unwrap();
            files.push(fs::read(&path).unwrap());
        }
        assert_eq!(files[0], files[1]);
        assert!(files[0].starts_with(b"\x89PNG"));
    }
}
