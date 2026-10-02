//! Making one thumbnail out of an image file.

use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{DynamicImage, ExtendedColorType, ImageEncoder, ImageReader};
use std::io::Cursor;
use std::path::Path;

/// The longest side of a thumbnail, in pixels.
pub(crate) const THUMBNAIL_SIDE: u32 = 128;
/// The quality opaque thumbnails are encoded at.
const JPEG_QUALITY: u8 = 85;

/// What became of an image file.
pub(crate) enum Made {
    /// The encoded thumbnail and its width and height in pixels.
    Thumbnail {
        /// PNG or JPEG bytes.
        bytes: Vec<u8>,
        /// Its width in pixels.
        width: u16,
        /// Its height in pixels.
        height: u16,
    },
    /// The file is not an image that can be decoded, for this reason.
    Broken(String),
}

/// Reads the file at `path` and makes its thumbnail: the first frame, decoded under the `image`
/// crate's default memory limit with no orientation applied, fitted with a box filter so its
/// longer side is at most [`THUMBNAIL_SIDE`] and never larger than the image, encoded as PNG when
/// any pixel is not fully opaque and as JPEG otherwise.
///
/// # Errors
///
/// The error of reading the file; a file that is read but not decoded is [`Made::Broken`].
pub(crate) fn make(path: &Path) -> std::io::Result<Made> {
    let bytes = std::fs::read(path)?;
    let decoded = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|error| error.to_string())
        .and_then(|reader| reader.decode().map_err(|error| error.to_string()));
    Ok(match decoded {
        Ok(image) => match thumbnail(&image) {
            Ok(made) => made,
            Err(reason) => Made::Broken(reason),
        },
        Err(reason) => Made::Broken(reason),
    })
}

/// The thumbnail of a decoded image.
///
/// # Errors
///
/// Why it could not be encoded.
fn thumbnail(image: &DynamicImage) -> Result<Made, String> {
    let (width, height) = (image.width(), image.height());
    let longer = width.max(height);
    let fitted = if longer > THUMBNAIL_SIDE {
        let scale = |side: u32| {
            let scaled = (u64::from(side) * u64::from(THUMBNAIL_SIDE) + u64::from(longer) / 2)
                / u64::from(longer);
            u32::try_from(scaled).unwrap_or(THUMBNAIL_SIDE).max(1)
        };
        image.thumbnail_exact(scale(width), scale(height))
    } else {
        image.clone()
    };
    let rgba = fitted.to_rgba8();
    let (width, height) = rgba.dimensions();
    let mut bytes = Vec::new();
    if rgba.pixels().any(|pixel| pixel.0[3] < u8::MAX) {
        PngEncoder::new(&mut bytes)
            .write_image(rgba.as_raw(), width, height, ExtendedColorType::Rgba8)
            .map_err(|error| error.to_string())?;
    } else {
        let rgb = DynamicImage::ImageRgba8(rgba).to_rgb8();
        JpegEncoder::new_with_quality(&mut bytes, JPEG_QUALITY)
            .encode_image(&rgb)
            .map_err(|error| error.to_string())?;
    }
    let side = |value: u32| u16::try_from(value).map_err(|error| error.to_string());
    Ok(Made::Thumbnail {
        bytes,
        width: side(width)?,
        height: side(height)?,
    })
}
