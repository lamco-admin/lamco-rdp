//! RDP file descriptor types for CLIPRDR file transfer.
//!
//! These types handle parsing and building FILEDESCRIPTORW structures
//! used in clipboard file transfer operations.

use lamco_clipboard_core::{ClipboardError, ClipboardResult};

/// Windows file descriptor flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileDescriptorFlags(u32);

impl FileDescriptorFlags {
    /// File attributes are present
    pub const ATTRIBUTES: u32 = 0x00000001;
    /// File size is present
    pub const FILESIZE: u32 = 0x00000040;
    /// Write time is present
    pub const WRITESTIME: u32 = 0x00000020;
    /// Creation time is present
    pub const CREATETIME: u32 = 0x00000002;
    /// Access time is present
    pub const ACCESSTIME: u32 = 0x00000010;

    /// Create from raw flags value
    pub fn from_raw(flags: u32) -> Self {
        Self(flags)
    }

    /// Check if a flag is set
    pub fn has_flag(&self, flag: u32) -> bool {
        (self.0 & flag) != 0
    }
}

/// File descriptor from FileGroupDescriptorW structure
///
/// Represents a single file in a clipboard file transfer operation.
/// Parsed from the 88-byte FILEDESCRIPTORW Windows structure.
#[derive(Debug, Clone)]
pub struct FileDescriptor {
    /// File descriptor flags indicating which fields are valid
    pub flags: FileDescriptorFlags,

    /// File attributes (Windows FILE_ATTRIBUTE_*)
    pub attributes: u32,

    /// File creation time (Windows FILETIME format - 100ns intervals since 1601-01-01)
    pub creation_time: Option<u64>,

    /// File last access time
    pub access_time: Option<u64>,

    /// File last write time
    pub write_time: Option<u64>,

    /// File size in bytes
    pub size: Option<u64>,

    /// File name (UTF-16 decoded to UTF-8, max 260 characters)
    pub name: String,
}

impl FileDescriptor {
    /// Parse a single FILEDESCRIPTORW structure from bytes
    ///
    /// # Format (592 bytes total)
    /// ```text
    /// Offset | Size | Field
    /// -------|------|------
    /// 0      | 4    | dwFlags
    /// 4      | 16   | clsid (GUID, unused)
    /// 20     | 8    | sizel (SIZE, unused)
    /// 28     | 8    | pointl (POINT, unused)
    /// 36     | 4    | dwFileAttributes
    /// 40     | 8    | ftCreationTime
    /// 48     | 8    | ftLastAccessTime
    /// 56     | 8    | ftLastWriteTime
    /// 64     | 8    | nFileSize (split into High:Low)
    /// 72     | 520  | cFileName (UTF-16, 260 chars max)
    /// ```
    pub fn parse(data: &[u8]) -> ClipboardResult<Self> {
        if data.len() < 592 {
            return Err(ClipboardError::FormatConversion(format!(
                "FILEDESCRIPTORW too small: {} bytes (need 592)",
                data.len()
            )));
        }

        let flags = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let flags = FileDescriptorFlags::from_raw(flags);

        let attributes = u32::from_le_bytes([data[36], data[37], data[38], data[39]]);

        let creation_time = if flags.has_flag(FileDescriptorFlags::CREATETIME) {
            Some(u64::from_le_bytes([
                data[40], data[41], data[42], data[43], data[44], data[45], data[46], data[47],
            ]))
        } else {
            None
        };

        let access_time = if flags.has_flag(FileDescriptorFlags::ACCESSTIME) {
            Some(u64::from_le_bytes([
                data[48], data[49], data[50], data[51], data[52], data[53], data[54], data[55],
            ]))
        } else {
            None
        };

        let write_time = if flags.has_flag(FileDescriptorFlags::WRITESTIME) {
            Some(u64::from_le_bytes([
                data[56], data[57], data[58], data[59], data[60], data[61], data[62], data[63],
            ]))
        } else {
            None
        };

        let size = if flags.has_flag(FileDescriptorFlags::FILESIZE) {
            let size_high = u32::from_le_bytes([data[64], data[65], data[66], data[67]]);
            let size_low = u32::from_le_bytes([data[68], data[69], data[70], data[71]]);
            Some(((size_high as u64) << 32) | (size_low as u64))
        } else {
            None
        };

        let filename_bytes = &data[72..592];
        let name = Self::parse_utf16_filename(filename_bytes)?;

        Ok(FileDescriptor {
            flags,
            attributes,
            creation_time,
            access_time,
            write_time,
            size,
            name,
        })
    }

    /// Parse UTF-16LE filename from raw bytes
    fn parse_utf16_filename(data: &[u8]) -> ClipboardResult<String> {
        if data.len() % 2 != 0 {
            return Err(ClipboardError::InvalidUtf16);
        }

        let utf16: Vec<u16> = data
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .take_while(|&c| c != 0)
            .collect();

        String::from_utf16(&utf16).map_err(|_| ClipboardError::InvalidUtf16)
    }

    /// Parse a list of file descriptors from FileGroupDescriptorW data
    ///
    /// # Format
    /// ```text
    /// Offset | Size | Field
    /// -------|------|------
    /// 0      | 4    | cItems (number of descriptors)
    /// 4      | 592  | fgd[0] (first FILEDESCRIPTORW)
    /// 596    | 592  | fgd[1] (second FILEDESCRIPTORW)
    /// ...
    /// ```
    pub fn parse_list(data: &[u8]) -> ClipboardResult<Vec<Self>> {
        if data.len() < 4 {
            return Err(ClipboardError::FormatConversion(
                "FileGroupDescriptorW too small for count".to_string(),
            ));
        }

        let count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;

        if count == 0 {
            return Ok(Vec::new());
        }

        let expected_size = 4 + (count * 592);
        if data.len() < expected_size {
            return Err(ClipboardError::FormatConversion(format!(
                "FileGroupDescriptorW too small: {} bytes (need {} for {} files)",
                data.len(),
                expected_size,
                count
            )));
        }

        let mut descriptors = Vec::with_capacity(count);
        for i in 0..count {
            let offset = 4 + (i * 592);
            let descriptor_data = &data[offset..offset + 592];
            descriptors.push(Self::parse(descriptor_data)?);
        }

        Ok(descriptors)
    }

    /// Build a single FILEDESCRIPTORW structure for a file
    ///
    /// Returns 592 bytes representing the file descriptor.
    /// The filename is sanitized for Windows compatibility.
    pub fn build(path: &std::path::Path) -> ClipboardResult<Vec<u8>> {
        let metadata = std::fs::metadata(path)
            .map_err(|e| ClipboardError::FormatConversion(format!("Failed to get file metadata: {}", e)))?;

        let raw_filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| ClipboardError::FormatConversion("Invalid filename".to_string()))?;

        let filename = lamco_clipboard_core::sanitize::sanitize_filename_for_windows(raw_filename);

        let mut data = vec![0u8; 592];

        let flags = FileDescriptorFlags::FILESIZE;
        data[0..4].copy_from_slice(&flags.to_le_bytes());

        let attributes: u32 = if metadata.is_dir() { 0x10 } else { 0x80 };
        data[36..40].copy_from_slice(&attributes.to_le_bytes());

        let size = metadata.len();
        let size_high = (size >> 32) as u32;
        let size_low = size as u32;
        data[64..68].copy_from_slice(&size_high.to_le_bytes());
        data[68..72].copy_from_slice(&size_low.to_le_bytes());

        let filename_utf16: Vec<u16> = filename.encode_utf16().collect();
        let filename_len = filename_utf16.len().min(259);
        for (i, &c) in filename_utf16.iter().take(filename_len).enumerate() {
            let offset = 72 + i * 2;
            data[offset..offset + 2].copy_from_slice(&c.to_le_bytes());
        }

        Ok(data)
    }

    /// Build FileGroupDescriptorW data from a list of file paths
    ///
    /// # Format
    /// ```text
    /// Offset | Size | Field
    /// -------|------|------
    /// 0      | 4    | cItems (number of descriptors)
    /// 4      | 592  | fgd[0] (first FILEDESCRIPTORW)
    /// 596    | 592  | fgd[1] (second FILEDESCRIPTORW)
    /// ...
    /// ```
    pub fn build_list(paths: &[std::path::PathBuf]) -> ClipboardResult<Vec<u8>> {
        let count = paths.len() as u32;
        let mut data = Vec::with_capacity(4 + paths.len() * 592);

        data.extend_from_slice(&count.to_le_bytes());

        for path in paths {
            let descriptor = Self::build(path)?;
            data.extend_from_slice(&descriptor);
        }

        Ok(data)
    }
}

/// Build FileGroupDescriptorW data from a list of file paths
///
/// This is a convenience function that calls FileDescriptor::build_list.
pub fn build_file_group_descriptor_w(paths: &[std::path::PathBuf]) -> ClipboardResult<Vec<u8>> {
    FileDescriptor::build_list(paths)
}
