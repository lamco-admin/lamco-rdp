//! Clipboard format conversion utilities.
//!
//! This module handles conversion between MIME types and Windows clipboard format IDs,
//! as well as data conversion between formats.

use crate::{ClipboardError, ClipboardResult};

// =============================================================================
// Windows Clipboard Format IDs
// =============================================================================

/// Standard Windows clipboard format: Unicode text (UTF-16LE)
pub const CF_UNICODETEXT: u32 = 13;

/// Standard Windows clipboard format: ANSI text
pub const CF_TEXT: u32 = 1;

/// Standard Windows clipboard format: Device-independent bitmap
pub const CF_DIB: u32 = 8;

/// Standard Windows clipboard format: File drop list
pub const CF_HDROP: u32 = 15;

/// Standard Windows clipboard format: Wave audio
pub const CF_WAVE: u32 = 12;

/// Standard Windows clipboard format: RIFF audio
pub const CF_RIFF: u32 = 11;

/// Custom format: HTML (registered format name: "HTML Format")
pub const CF_HTML: u32 = 0xD010;

/// Custom format: PNG image
pub const CF_PNG: u32 = 0xD011;

/// Custom format: JPEG image
pub const CF_JPEG: u32 = 0xD012;

/// Custom format: GIF image
pub const CF_GIF: u32 = 0xD013;

/// Custom format: Rich Text Format
pub const CF_RTF: u32 = 0xD014;

// =============================================================================
// Clipboard Format
// =============================================================================

/// A clipboard format with ID and optional name
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClipboardFormat {
    /// Windows clipboard format ID
    pub id: u32,

    /// Format name (for registered formats)
    pub name: Option<String>,
}

impl ClipboardFormat {
    /// Create a new clipboard format with ID only
    pub fn new(id: u32) -> Self {
        Self { id, name: None }
    }

    /// Create a new clipboard format with ID and name
    pub fn with_name(id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: Some(name.into()),
        }
    }

    /// Create format for Unicode text
    pub fn unicode_text() -> Self {
        Self::new(CF_UNICODETEXT)
    }

    /// Create format for HTML
    pub fn html() -> Self {
        Self::with_name(CF_HTML, "HTML Format")
    }

    /// Create format for PNG
    pub fn png() -> Self {
        Self::with_name(CF_PNG, "PNG")
    }

    /// Create format for file drop
    pub fn file_drop() -> Self {
        Self::new(CF_HDROP)
    }
}

// =============================================================================
// MIME <-> Format Conversion
// =============================================================================

/// Convert MIME types to RDP clipboard formats
///
/// # Example
///
/// ```
/// use lamco_clipboard_core::formats::mime_to_rdp_formats;
///
/// let formats = mime_to_rdp_formats(&["text/plain", "text/html"]);
/// assert!(!formats.is_empty());
/// ```
pub fn mime_to_rdp_formats(mime_types: &[&str]) -> Vec<ClipboardFormat> {
    let mut formats = Vec::new();

    for mime in mime_types {
        match *mime {
            // Text formats
            "text/plain" | "text/plain;charset=utf-8" | "UTF8_STRING" | "STRING" => {
                if !formats.iter().any(|f: &ClipboardFormat| f.id == CF_UNICODETEXT) {
                    formats.push(ClipboardFormat::unicode_text());
                }
            }

            "text/html" => {
                formats.push(ClipboardFormat::html());
            }

            "text/rtf" | "application/rtf" => {
                formats.push(ClipboardFormat::with_name(CF_RTF, "Rich Text Format"));
            }

            // Image formats
            "image/png" => {
                formats.push(ClipboardFormat::png());
                // Also offer DIB for compatibility
                if !formats.iter().any(|f: &ClipboardFormat| f.id == CF_DIB) {
                    formats.push(ClipboardFormat::new(CF_DIB));
                }
            }

            "image/jpeg" | "image/jpg" => {
                formats.push(ClipboardFormat::with_name(CF_JPEG, "JFIF"));
                if !formats.iter().any(|f: &ClipboardFormat| f.id == CF_DIB) {
                    formats.push(ClipboardFormat::new(CF_DIB));
                }
            }

            "image/gif" => {
                formats.push(ClipboardFormat::with_name(CF_GIF, "GIF"));
            }

            "image/bmp" | "image/x-bmp" => {
                formats.push(ClipboardFormat::new(CF_DIB));
            }

            // File formats
            "text/uri-list" | "x-special/gnome-copied-files" => {
                if !formats.iter().any(|f: &ClipboardFormat| f.id == CF_HDROP) {
                    formats.push(ClipboardFormat::file_drop());
                }
            }

            // Audio formats
            "audio/wav" | "audio/x-wav" => {
                formats.push(ClipboardFormat::new(CF_WAVE));
            }

            _ => {
                // Unknown format - skip
                tracing::debug!("Unknown MIME type: {}", mime);
            }
        }
    }

    formats
}

/// Convert RDP format ID to preferred MIME type
///
/// # Example
///
/// ```
/// use lamco_clipboard_core::formats::{rdp_format_to_mime, CF_UNICODETEXT};
///
/// let mime = rdp_format_to_mime(CF_UNICODETEXT);
/// assert_eq!(mime, Some("text/plain;charset=utf-8"));
/// ```
pub fn rdp_format_to_mime(format_id: u32) -> Option<&'static str> {
    match format_id {
        CF_UNICODETEXT | CF_TEXT => Some("text/plain;charset=utf-8"),
        CF_HTML => Some("text/html"),
        CF_RTF => Some("text/rtf"),
        CF_DIB => Some("image/png"), // Prefer PNG output
        CF_PNG => Some("image/png"),
        CF_JPEG => Some("image/jpeg"),
        CF_GIF => Some("image/gif"),
        CF_HDROP => Some("text/uri-list"),
        CF_WAVE | CF_RIFF => Some("audio/wav"),
        _ => None,
    }
}

// =============================================================================
// Format Converter
// =============================================================================

/// Handles clipboard data format conversion
#[derive(Debug, Default)]
pub struct FormatConverter {
    /// Maximum data size for conversion (default: 16MB)
    pub max_size: usize,
}

impl FormatConverter {
    /// Create a new format converter with default settings
    pub fn new() -> Self {
        Self {
            max_size: 16 * 1024 * 1024, // 16MB
        }
    }

    /// Create a format converter with custom max size
    pub fn with_max_size(max_size: usize) -> Self {
        Self { max_size }
    }

    /// Convert UTF-8 text to UTF-16LE (for CF_UNICODETEXT)
    ///
    /// Adds null terminator as required by Windows.
    pub fn text_to_unicode(&self, text: &str) -> ClipboardResult<Vec<u8>> {
        if text.len() > self.max_size {
            return Err(ClipboardError::DataSizeExceeded {
                actual: text.len(),
                max: self.max_size,
            });
        }

        let mut result: Vec<u8> = text.encode_utf16().flat_map(|c| c.to_le_bytes()).collect();

        // Add null terminator (2 bytes for UTF-16)
        result.extend_from_slice(&[0, 0]);

        Ok(result)
    }

    /// Convert UTF-16LE to UTF-8 (from CF_UNICODETEXT)
    pub fn unicode_to_text(&self, data: &[u8]) -> ClipboardResult<String> {
        if data.len() > self.max_size {
            return Err(ClipboardError::DataSizeExceeded {
                actual: data.len(),
                max: self.max_size,
            });
        }

        if data.len() % 2 != 0 {
            return Err(ClipboardError::InvalidUtf16);
        }

        let utf16: Vec<u16> = data
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        // Remove null terminator if present
        let utf16 = if utf16.last() == Some(&0) {
            &utf16[..utf16.len() - 1]
        } else {
            &utf16[..]
        };

        String::from_utf16(utf16).map_err(|_| ClipboardError::InvalidUtf16)
    }

    /// Convert plain HTML to Windows CF_HTML format
    ///
    /// The CF_HTML format includes headers with byte offsets.
    pub fn html_to_cf_html(&self, html: &str) -> ClipboardResult<Vec<u8>> {
        if html.len() > self.max_size {
            return Err(ClipboardError::DataSizeExceeded {
                actual: html.len(),
                max: self.max_size,
            });
        }

        // CF_HTML format:
        // Version:0.9
        // StartHTML:XXXXXXXX
        // EndHTML:XXXXXXXX
        // StartFragment:XXXXXXXX
        // EndFragment:XXXXXXXX
        // <html><body><!--StartFragment-->CONTENT<!--EndFragment--></body></html>

        let header_template = "Version:0.9\r\n\
                               StartHTML:XXXXXXXX\r\n\
                               EndHTML:XXXXXXXX\r\n\
                               StartFragment:XXXXXXXX\r\n\
                               EndFragment:XXXXXXXX\r\n";

        let prefix = "<html><body><!--StartFragment-->";
        let suffix = "<!--EndFragment--></body></html>";

        let header_len = header_template.len();
        let start_html = header_len;
        let start_fragment = header_len + prefix.len();
        let end_fragment = start_fragment + html.len();
        let end_html = end_fragment + suffix.len();

        let header = format!(
            "Version:0.9\r\n\
             StartHTML:{:08}\r\n\
             EndHTML:{:08}\r\n\
             StartFragment:{:08}\r\n\
             EndFragment:{:08}\r\n",
            start_html, end_html, start_fragment, end_fragment
        );

        let mut result = header;
        result.push_str(prefix);
        result.push_str(html);
        result.push_str(suffix);

        Ok(result.into_bytes())
    }

    /// Extract HTML content from CF_HTML format
    pub fn cf_html_to_html(&self, data: &[u8]) -> ClipboardResult<String> {
        let text = std::str::from_utf8(data).map_err(|_| ClipboardError::InvalidUtf8)?;

        // Parse StartFragment and EndFragment from header
        let start_fragment = Self::parse_header_value(text, "StartFragment:")?;
        let end_fragment = Self::parse_header_value(text, "EndFragment:")?;

        if start_fragment >= end_fragment || end_fragment > data.len() {
            return Err(ClipboardError::FormatConversion("invalid CF_HTML offsets".to_string()));
        }

        let fragment = &text[start_fragment..end_fragment];
        Ok(fragment.to_string())
    }

    /// Parse a numeric header value from CF_HTML
    fn parse_header_value(text: &str, key: &str) -> ClipboardResult<usize> {
        text.lines()
            .find(|line| line.starts_with(key))
            .and_then(|line| line[key.len()..].trim().parse().ok())
            .ok_or_else(|| ClipboardError::FormatConversion(format!("missing {} header", key)))
    }

    /// Convert URI list to HDROP format (file paths)
    ///
    /// The HDROP format is a DROPFILES structure followed by null-terminated paths.
    pub fn uri_list_to_hdrop(&self, uri_list: &str) -> ClipboardResult<Vec<u8>> {
        let paths: Vec<&str> = uri_list
            .lines()
            .filter(|line| !line.starts_with('#'))
            .filter_map(|line| line.strip_prefix("file://"))
            .collect();

        if paths.is_empty() {
            return Err(ClipboardError::FormatConversion("no valid file URIs".to_string()));
        }

        // DROPFILES structure (20 bytes):
        // DWORD pFiles (offset to file list)
        // POINT pt (unused, 8 bytes)
        // BOOL fNC (unused, 4 bytes)
        // BOOL fWide (TRUE for Unicode)

        let mut result = Vec::new();

        // pFiles: offset 20 (size of DROPFILES)
        result.extend_from_slice(&20u32.to_le_bytes());
        // pt.x, pt.y (unused)
        result.extend_from_slice(&0i32.to_le_bytes());
        result.extend_from_slice(&0i32.to_le_bytes());
        // fNC (unused)
        result.extend_from_slice(&0u32.to_le_bytes());
        // fWide = TRUE (Unicode paths)
        result.extend_from_slice(&1u32.to_le_bytes());

        // File paths as UTF-16LE, null-terminated
        for path in paths {
            // URL decode the path
            let decoded = percent_decode(path);
            for c in decoded.encode_utf16() {
                result.extend_from_slice(&c.to_le_bytes());
            }
            // Null terminator
            result.extend_from_slice(&[0, 0]);
        }

        // Final double null terminator
        result.extend_from_slice(&[0, 0]);

        Ok(result)
    }

    /// Convert HDROP format to URI list
    pub fn hdrop_to_uri_list(&self, data: &[u8]) -> ClipboardResult<String> {
        if data.len() < 20 {
            return Err(ClipboardError::FormatConversion("HDROP too small".to_string()));
        }

        // Read DROPFILES header
        let p_files = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let f_wide = u32::from_le_bytes([data[16], data[17], data[18], data[19]]) != 0;

        if p_files >= data.len() {
            return Err(ClipboardError::FormatConversion("invalid pFiles offset".to_string()));
        }

        let mut paths = Vec::new();
        let file_data = &data[p_files..];

        if f_wide {
            // UTF-16LE paths
            let mut pos = 0;
            while pos + 2 <= file_data.len() {
                let mut path_chars = Vec::new();
                while pos + 2 <= file_data.len() {
                    let c = u16::from_le_bytes([file_data[pos], file_data[pos + 1]]);
                    pos += 2;
                    if c == 0 {
                        break;
                    }
                    path_chars.push(c);
                }

                if path_chars.is_empty() {
                    break; // Double null = end
                }

                if let Ok(path) = String::from_utf16(&path_chars) {
                    paths.push(format!("file://{}", percent_encode(&path)));
                }
            }
        } else {
            // ANSI paths (rare)
            let mut pos = 0;
            while pos < file_data.len() {
                let end = file_data[pos..]
                    .iter()
                    .position(|&b| b == 0)
                    .unwrap_or(file_data.len() - pos);
                if end == 0 {
                    break;
                }
                if let Ok(path) = std::str::from_utf8(&file_data[pos..pos + end]) {
                    paths.push(format!("file://{}", percent_encode(path)));
                }
                pos += end + 1;
            }
        }

        Ok(paths.join("\r\n"))
    }
}

// =============================================================================
// URL Encoding Helpers
// =============================================================================

/// Percent-decode a URL path
fn percent_decode(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Percent-encode special characters in a path
fn percent_encode(input: &str) -> String {
    let mut result = String::new();

    for c in input.chars() {
        match c {
            ' ' => result.push_str("%20"),
            '#' => result.push_str("%23"),
            '%' => result.push_str("%25"),
            '?' => result.push_str("%3F"),
            _ => result.push(c),
        }
    }

    result
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mime_to_formats() {
        let formats = mime_to_rdp_formats(&["text/plain", "text/html"]);
        assert!(formats.iter().any(|f| f.id == CF_UNICODETEXT));
        assert!(formats.iter().any(|f| f.id == CF_HTML));
    }

    #[test]
    fn test_format_to_mime() {
        assert_eq!(rdp_format_to_mime(CF_UNICODETEXT), Some("text/plain;charset=utf-8"));
        assert_eq!(rdp_format_to_mime(CF_HTML), Some("text/html"));
        assert_eq!(rdp_format_to_mime(CF_PNG), Some("image/png"));
        assert_eq!(rdp_format_to_mime(0xFFFF), None);
    }

    #[test]
    fn test_text_to_unicode() {
        let converter = FormatConverter::new();
        let result = converter.text_to_unicode("Hello").unwrap();

        // "Hello" in UTF-16LE + null terminator
        assert_eq!(
            result,
            vec![
                b'H', 0, b'e', 0, b'l', 0, b'l', 0, b'o', 0, // "Hello"
                0, 0 // null terminator
            ]
        );
    }

    #[test]
    fn test_unicode_to_text() {
        let converter = FormatConverter::new();
        let data = vec![b'H', 0, b'i', 0, 0, 0]; // "Hi" + null
        let result = converter.unicode_to_text(&data).unwrap();
        assert_eq!(result, "Hi");
    }

    #[test]
    fn test_html_roundtrip() {
        let converter = FormatConverter::new();
        let html = "<b>Hello</b>";

        let cf_html = converter.html_to_cf_html(html).unwrap();
        let recovered = converter.cf_html_to_html(&cf_html).unwrap();

        assert_eq!(recovered, html);
    }

    #[test]
    fn test_clipboard_format_builders() {
        let text = ClipboardFormat::unicode_text();
        assert_eq!(text.id, CF_UNICODETEXT);
        assert!(text.name.is_none());

        let html = ClipboardFormat::html();
        assert_eq!(html.id, CF_HTML);
        assert_eq!(html.name, Some("HTML Format".to_string()));
    }

    #[test]
    fn test_uri_list_to_hdrop() {
        let converter = FormatConverter::new();
        let uri_list = "file:///home/user/test.txt";

        let hdrop = converter.uri_list_to_hdrop(uri_list).unwrap();

        // Check DROPFILES header
        assert_eq!(hdrop[0..4], 20u32.to_le_bytes()); // pFiles
        assert_eq!(hdrop[16..20], 1u32.to_le_bytes()); // fWide = TRUE
    }

    #[test]
    fn test_hdrop_roundtrip() {
        let converter = FormatConverter::new();
        let original = "file:///home/user/test.txt";

        let hdrop = converter.uri_list_to_hdrop(original).unwrap();
        let recovered = converter.hdrop_to_uri_list(&hdrop).unwrap();

        assert_eq!(recovered, original);
    }
}
