//! Image format conversion utilities.
//!
//! This module provides conversion between image formats commonly used in
//! clipboard operations, particularly the Windows DIB (Device Independent Bitmap)
//! format and standard image formats like PNG and JPEG.
//!
//! # Feature Flag
//!
//! This module requires the `image` feature:
//!
//! ```toml
//! [dependencies]
//! lamco-clipboard-core = { version = "0.1", features = ["image"] }
//! ```
//!
//! # Supported Conversions
//!
//! - PNG ↔ DIB
//! - JPEG ↔ DIB
//! - BMP ↔ DIB
//! - GIF → PNG (read-only, converts to PNG for output)

use bytes::{BufMut, BytesMut};
use image::{DynamicImage, ImageFormat};

use crate::{ClipboardError, ClipboardResult};

/// Convert PNG image data to DIB (Device Independent Bitmap) format.
///
/// DIB is the standard Windows bitmap format used in clipboard operations.
/// This function decodes the PNG and creates a 32-bit BGRA DIB.
///
/// # Example
///
/// ```ignore
/// use lamco_clipboard_core::image::png_to_dib;
///
/// let png_data = std::fs::read("image.png")?;
/// let dib_data = png_to_dib(&png_data)?;
/// ```
pub fn png_to_dib(png_data: &[u8]) -> ClipboardResult<Vec<u8>> {
    let image = image::load_from_memory_with_format(png_data, ImageFormat::Png)
        .map_err(|e| ClipboardError::ImageDecode(e.to_string()))?;

    create_dib_from_image(&image)
}

/// Convert JPEG image data to DIB format.
pub fn jpeg_to_dib(jpeg_data: &[u8]) -> ClipboardResult<Vec<u8>> {
    let image = image::load_from_memory_with_format(jpeg_data, ImageFormat::Jpeg)
        .map_err(|e| ClipboardError::ImageDecode(e.to_string()))?;

    create_dib_from_image(&image)
}

/// Convert GIF image data to DIB format.
///
/// Note: GIF animations are not supported; only the first frame is converted.
pub fn gif_to_dib(gif_data: &[u8]) -> ClipboardResult<Vec<u8>> {
    let image = image::load_from_memory_with_format(gif_data, ImageFormat::Gif)
        .map_err(|e| ClipboardError::ImageDecode(e.to_string()))?;

    create_dib_from_image(&image)
}

/// Convert BMP file data to DIB format.
///
/// BMP files have a 14-byte file header followed by the DIB data.
/// This function extracts the DIB portion.
pub fn bmp_to_dib(bmp_data: &[u8]) -> ClipboardResult<Vec<u8>> {
    if bmp_data.len() < 14 {
        return Err(ClipboardError::ImageDecode("BMP file too small".to_string()));
    }

    // Verify BMP signature
    if &bmp_data[0..2] != b"BM" {
        return Err(ClipboardError::ImageDecode("Invalid BMP signature".to_string()));
    }

    // DIB is everything after the 14-byte file header
    Ok(bmp_data[14..].to_vec())
}

/// Convert DIB data to PNG format.
///
/// This is the most common conversion for clipboard images going from
/// Windows to Linux, as PNG is widely supported and lossless.
pub fn dib_to_png(dib_data: &[u8]) -> ClipboardResult<Vec<u8>> {
    let image = parse_dib_to_image(dib_data)?;

    let mut png_data = Vec::new();
    image
        .write_to(&mut std::io::Cursor::new(&mut png_data), ImageFormat::Png)
        .map_err(|e| ClipboardError::ImageEncode(e.to_string()))?;

    Ok(png_data)
}

/// Convert DIB data to JPEG format.
///
/// JPEG is lossy but produces smaller files. Use for photographs.
pub fn dib_to_jpeg(dib_data: &[u8]) -> ClipboardResult<Vec<u8>> {
    let image = parse_dib_to_image(dib_data)?;

    let mut jpeg_data = Vec::new();
    image
        .write_to(&mut std::io::Cursor::new(&mut jpeg_data), ImageFormat::Jpeg)
        .map_err(|e| ClipboardError::ImageEncode(e.to_string()))?;

    Ok(jpeg_data)
}

/// Convert DIB data to BMP file format.
///
/// This adds the 14-byte BMP file header to the DIB data.
pub fn dib_to_bmp(dib_data: &[u8]) -> ClipboardResult<Vec<u8>> {
    if dib_data.len() < 40 {
        return Err(ClipboardError::ImageDecode("DIB too small".to_string()));
    }

    // Parse DIB header to calculate file size
    let file_size =
        u32::try_from(14 + dib_data.len()).map_err(|_| ClipboardError::ImageDecode("DIB too large".to_string()))?;
    let pixel_offset: u32 = 14 + 40; // File header + DIB header (minimum)

    let mut bmp = BytesMut::new();

    // BMP file header (14 bytes)
    bmp.put_slice(b"BM"); // Signature
    bmp.put_u32_le(file_size); // File size
    bmp.put_u16_le(0); // Reserved1
    bmp.put_u16_le(0); // Reserved2
    bmp.put_u32_le(pixel_offset); // Pixel data offset

    // Append DIB data
    bmp.put_slice(dib_data);

    Ok(bmp.to_vec())
}

/// Convert any supported image format to DIB.
///
/// Automatically detects the input format based on magic bytes.
pub fn any_to_dib(data: &[u8]) -> ClipboardResult<Vec<u8>> {
    let image = image::load_from_memory(data).map_err(|e| ClipboardError::ImageDecode(e.to_string()))?;

    create_dib_from_image(&image)
}

/// Get image dimensions from DIB data without full decode.
///
/// Returns (width, height) in pixels.
pub fn dib_dimensions(dib_data: &[u8]) -> ClipboardResult<(u32, u32)> {
    if dib_data.len() < 12 {
        return Err(ClipboardError::ImageDecode("DIB too small".to_string()));
    }

    let width = i32::from_le_bytes([dib_data[4], dib_data[5], dib_data[6], dib_data[7]]).unsigned_abs();
    let height = i32::from_le_bytes([dib_data[8], dib_data[9], dib_data[10], dib_data[11]]).unsigned_abs();

    Ok((width, height))
}

// =============================================================================
// Internal Functions
// =============================================================================

/// Create DIB data from a DynamicImage.
fn create_dib_from_image(image: &DynamicImage) -> ClipboardResult<Vec<u8>> {
    let rgba = image.to_rgba8();
    let (width, height) = (rgba.width(), rgba.height());

    let mut dib = BytesMut::new();

    // BITMAPINFOHEADER structure (40 bytes)
    dib.put_u32_le(40); // biSize
    dib.put_i32_le(i32::try_from(width).unwrap_or(i32::MAX)); // biWidth
    dib.put_i32_le(-i32::try_from(height).unwrap_or(i32::MAX)); // biHeight (negative for top-down)
    dib.put_u16_le(1); // biPlanes
    dib.put_u16_le(32); // biBitCount (32 bits for BGRA)
    dib.put_u32_le(0); // biCompression (BI_RGB = 0)

    let image_size = width.saturating_mul(height).saturating_mul(4);
    dib.put_u32_le(image_size); // biSizeImage

    dib.put_i32_le(0); // biXPelsPerMeter
    dib.put_i32_le(0); // biYPelsPerMeter
    dib.put_u32_le(0); // biClrUsed
    dib.put_u32_le(0); // biClrImportant

    // Pixel data (convert RGBA to BGRA - Windows byte order)
    for pixel in rgba.pixels() {
        dib.put_u8(pixel[2]); // Blue
        dib.put_u8(pixel[1]); // Green
        dib.put_u8(pixel[0]); // Red
        dib.put_u8(pixel[3]); // Alpha
    }

    Ok(dib.to_vec())
}

/// Parse DIB data into a DynamicImage.
fn parse_dib_to_image(dib_data: &[u8]) -> ClipboardResult<DynamicImage> {
    if dib_data.len() < 40 {
        return Err(ClipboardError::ImageDecode("DIB too small".to_string()));
    }

    // Parse BITMAPINFOHEADER
    let bi_size = u32::from_le_bytes([dib_data[0], dib_data[1], dib_data[2], dib_data[3]]);
    if bi_size < 40 {
        return Err(ClipboardError::ImageDecode("Invalid DIB header size".to_string()));
    }

    let width = i32::from_le_bytes([dib_data[4], dib_data[5], dib_data[6], dib_data[7]]).unsigned_abs();
    let height_raw = i32::from_le_bytes([dib_data[8], dib_data[9], dib_data[10], dib_data[11]]);
    let height = height_raw.unsigned_abs();
    let top_down = height_raw < 0;
    let bit_count = u16::from_le_bytes([dib_data[14], dib_data[15]]);

    let header_size = bi_size as usize;
    if header_size >= dib_data.len() {
        return Err(ClipboardError::ImageDecode("DIB header larger than data".to_string()));
    }
    let pixel_data = &dib_data[header_size..];

    // Convert based on bit depth
    let image = match bit_count {
        32 => convert_32bit_dib(pixel_data, width, height, top_down)?,
        24 => convert_24bit_dib(pixel_data, width, height, top_down)?,
        _ => {
            return Err(ClipboardError::ImageDecode(format!(
                "Unsupported DIB bit depth: {}",
                bit_count
            )))
        }
    };

    Ok(image)
}

/// Convert 32-bit BGRA DIB to RGBA image.
fn convert_32bit_dib(pixel_data: &[u8], width: u32, height: u32, top_down: bool) -> ClipboardResult<DynamicImage> {
    let expected_size = (width as usize) * (height as usize) * 4;
    if pixel_data.len() < expected_size {
        return Err(ClipboardError::ImageDecode(format!(
            "Insufficient pixel data: {} < {}",
            pixel_data.len(),
            expected_size
        )));
    }

    let mut rgba_data = Vec::with_capacity(expected_size);

    for y in 0..height {
        let row_y = if top_down { y } else { height - 1 - y };
        let row_offset = (row_y as usize) * (width as usize) * 4;

        for x in 0..width {
            let pixel_offset = row_offset + (x as usize) * 4;
            if pixel_offset + 3 < pixel_data.len() {
                rgba_data.push(pixel_data[pixel_offset + 2]); // Red
                rgba_data.push(pixel_data[pixel_offset + 1]); // Green
                rgba_data.push(pixel_data[pixel_offset]); // Blue
                rgba_data.push(pixel_data[pixel_offset + 3]); // Alpha
            }
        }
    }

    image::RgbaImage::from_raw(width, height, rgba_data)
        .map(DynamicImage::ImageRgba8)
        .ok_or_else(|| ClipboardError::ImageDecode("Failed to create image from DIB".to_string()))
}

/// Convert 24-bit BGR DIB to RGB image.
fn convert_24bit_dib(pixel_data: &[u8], width: u32, height: u32, top_down: bool) -> ClipboardResult<DynamicImage> {
    // 24-bit DIB rows are aligned to 4-byte boundaries
    let row_size = (width * 3).div_ceil(4) * 4;
    let expected_size = (row_size as usize) * (height as usize);

    if pixel_data.len() < expected_size {
        return Err(ClipboardError::ImageDecode(format!(
            "Insufficient pixel data: {} < {}",
            pixel_data.len(),
            expected_size
        )));
    }

    let mut rgb_data = Vec::with_capacity((width as usize) * (height as usize) * 3);

    for y in 0..height {
        let row_y = if top_down { y } else { height - 1 - y };
        let row_offset = (row_y as usize) * (row_size as usize);

        for x in 0..width {
            let pixel_offset = row_offset + (x as usize) * 3;
            if pixel_offset + 2 < pixel_data.len() {
                rgb_data.push(pixel_data[pixel_offset + 2]); // Red
                rgb_data.push(pixel_data[pixel_offset + 1]); // Green
                rgb_data.push(pixel_data[pixel_offset]); // Blue
            }
        }
    }

    image::RgbImage::from_raw(width, height, rgb_data)
        .map(DynamicImage::ImageRgb8)
        .ok_or_else(|| ClipboardError::ImageDecode("Failed to create image from DIB".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_parse_dib() {
        // Create a small test image (10x10 red square)
        let image = DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(10, 10, image::Rgba([255, 0, 0, 255])));

        // Convert to DIB
        let dib = create_dib_from_image(&image).unwrap();

        // Verify DIB header
        assert!(dib.len() >= 40);
        assert_eq!(u32::from_le_bytes([dib[0], dib[1], dib[2], dib[3]]), 40); // biSize

        // Convert back to image
        let parsed = parse_dib_to_image(&dib).unwrap();
        assert_eq!(parsed.width(), 10);
        assert_eq!(parsed.height(), 10);
    }

    #[test]
    fn test_dib_dimensions() {
        let image = DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(100, 50, image::Rgba([0, 0, 0, 255])));

        let dib = create_dib_from_image(&image).unwrap();
        let (width, height) = dib_dimensions(&dib).unwrap();

        assert_eq!(width, 100);
        assert_eq!(height, 50);
    }

    #[test]
    fn test_png_roundtrip() {
        // Create a small PNG
        let image = DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(5, 5, image::Rgba([100, 150, 200, 255])));

        let mut png_data = Vec::new();
        image
            .write_to(&mut std::io::Cursor::new(&mut png_data), ImageFormat::Png)
            .unwrap();

        // PNG → DIB → PNG
        let dib = png_to_dib(&png_data).unwrap();
        let png_back = dib_to_png(&dib).unwrap();

        // Load and verify
        let loaded = image::load_from_memory(&png_back).unwrap();
        assert_eq!(loaded.width(), 5);
        assert_eq!(loaded.height(), 5);
    }

    #[test]
    fn test_bmp_roundtrip() {
        let image = DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(8, 8, image::Rgba([50, 100, 150, 255])));

        let dib = create_dib_from_image(&image).unwrap();

        // DIB → BMP → DIB
        let bmp = dib_to_bmp(&dib).unwrap();

        // Verify BMP signature
        assert_eq!(&bmp[0..2], b"BM");

        // Extract DIB back
        let dib_back = bmp_to_dib(&bmp).unwrap();
        assert_eq!(dib, dib_back);
    }

    #[test]
    fn test_invalid_dib() {
        // Too small
        assert!(parse_dib_to_image(&[0; 30]).is_err());

        // Invalid header size
        let mut invalid_dib = vec![0; 50];
        invalid_dib[0] = 10; // Invalid biSize < 40
        assert!(parse_dib_to_image(&invalid_dib).is_err());
    }

    #[test]
    fn test_invalid_bmp() {
        // Too small
        assert!(bmp_to_dib(&[0; 10]).is_err());

        // Invalid signature
        let mut invalid_bmp = vec![0; 20];
        invalid_bmp[0] = b'X';
        invalid_bmp[1] = b'Y';
        assert!(bmp_to_dib(&invalid_bmp).is_err());
    }
}
