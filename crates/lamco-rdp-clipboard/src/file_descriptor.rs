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
        if !data.len().is_multiple_of(2) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Assemble a raw 592-byte FILEDESCRIPTORW with the given flags and name.
    fn raw_descriptor(flags: u32, name: &str) -> Vec<u8> {
        let mut data = vec![0u8; 592];
        data[0..4].copy_from_slice(&flags.to_le_bytes());
        for (i, c) in name.encode_utf16().enumerate() {
            let off = 72 + i * 2;
            data[off..off + 2].copy_from_slice(&c.to_le_bytes());
        }
        data
    }

    #[test]
    fn parse_rejects_short_input() {
        assert!(FileDescriptor::parse(&[0u8; 591]).is_err());
        assert!(FileDescriptor::parse(&[]).is_err());
    }

    #[test]
    fn parse_minimal_no_flags_has_no_optional_fields() {
        let fd = FileDescriptor::parse(&raw_descriptor(0, "")).unwrap();
        assert_eq!(fd.name, "");
        assert!(fd.size.is_none());
        assert!(fd.creation_time.is_none());
        assert!(fd.access_time.is_none());
        assert!(fd.write_time.is_none());
    }

    #[test]
    fn parse_reads_utf16_name() {
        let fd = FileDescriptor::parse(&raw_descriptor(0, "report.pdf")).unwrap();
        assert_eq!(fd.name, "report.pdf");
    }

    #[test]
    fn parse_name_stops_at_null_terminator() {
        let mut data = raw_descriptor(0, "ab");
        // null at char index 2, then a stray 'X' that must be ignored.
        data[76..78].copy_from_slice(&0u16.to_le_bytes());
        data[78..80].copy_from_slice(&(u16::from(b'X')).to_le_bytes());
        let fd = FileDescriptor::parse(&data).unwrap();
        assert_eq!(fd.name, "ab");
    }

    #[test]
    fn parse_size_flag_combines_high_and_low_dwords() {
        let mut data = raw_descriptor(FileDescriptorFlags::FILESIZE, "f");
        data[64..68].copy_from_slice(&1u32.to_le_bytes()); // high dword
        data[68..72].copy_from_slice(&5u32.to_le_bytes()); // low dword
        let fd = FileDescriptor::parse(&data).unwrap();
        assert_eq!(fd.size, Some((1u64 << 32) | 5));
    }

    #[test]
    fn parse_time_flags_gate_their_fields() {
        let flags = FileDescriptorFlags::CREATETIME | FileDescriptorFlags::ACCESSTIME | FileDescriptorFlags::WRITESTIME;
        let mut data = raw_descriptor(flags, "f");
        data[40..48].copy_from_slice(&111u64.to_le_bytes());
        data[48..56].copy_from_slice(&222u64.to_le_bytes());
        data[56..64].copy_from_slice(&333u64.to_le_bytes());
        let fd = FileDescriptor::parse(&data).unwrap();
        assert_eq!(fd.creation_time, Some(111));
        assert_eq!(fd.access_time, Some(222));
        assert_eq!(fd.write_time, Some(333));
    }

    #[test]
    fn parse_utf16_filename_rejects_odd_length() {
        assert!(matches!(
            FileDescriptor::parse_utf16_filename(&[0x41]),
            Err(ClipboardError::InvalidUtf16)
        ));
    }

    #[test]
    fn parse_utf16_filename_rejects_lone_surrogate() {
        // 0xD800 is a high surrogate with no following low surrogate.
        assert!(matches!(
            FileDescriptor::parse_utf16_filename(&0xD800u16.to_le_bytes()),
            Err(ClipboardError::InvalidUtf16)
        ));
    }

    #[test]
    fn parse_list_empty_count_yields_no_descriptors() {
        assert!(FileDescriptor::parse_list(&[0u8; 4]).unwrap().is_empty());
    }

    #[test]
    fn parse_list_rejects_truncated_count_header() {
        assert!(FileDescriptor::parse_list(&[0u8; 3]).is_err());
    }

    #[test]
    fn parse_list_rejects_count_exceeding_data() {
        let mut data = vec![0u8; 4];
        data[0..4].copy_from_slice(&5u32.to_le_bytes()); // claims 5 files, no bodies
        assert!(FileDescriptor::parse_list(&data).is_err());
    }

    #[test]
    fn parse_list_huge_count_errors_without_allocating() {
        // Attacker-controlled count must hit the expected-size guard before the
        // Vec::with_capacity, so this returns Err rather than OOM-aborting.
        let mut data = vec![0u8; 4];
        data[0..4].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(FileDescriptor::parse_list(&data).is_err());
    }

    #[test]
    fn parse_list_reads_multiple_descriptors() {
        let mut data = 2u32.to_le_bytes().to_vec();
        data.extend_from_slice(&raw_descriptor(0, "first.txt"));
        data.extend_from_slice(&raw_descriptor(0, "second.txt"));
        let list = FileDescriptor::parse_list(&data).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "first.txt");
        assert_eq!(list[1].name, "second.txt");
    }

    #[test]
    fn flags_has_flag_detects_set_bits() {
        let f = FileDescriptorFlags::from_raw(FileDescriptorFlags::FILESIZE | FileDescriptorFlags::CREATETIME);
        assert!(f.has_flag(FileDescriptorFlags::FILESIZE));
        assert!(f.has_flag(FileDescriptorFlags::CREATETIME));
        assert!(!f.has_flag(FileDescriptorFlags::ACCESSTIME));
    }

    #[test]
    fn build_list_empty_is_just_a_zero_count() {
        assert_eq!(FileDescriptor::build_list(&[]).unwrap(), vec![0u8; 4]);
        assert_eq!(build_file_group_descriptor_w(&[]).unwrap(), vec![0u8; 4]);
    }

    #[test]
    fn build_then_parse_roundtrips_a_real_file() {
        let mut path = std::env::temp_dir();
        path.push(format!("lamco_fd_roundtrip_{}.txt", std::process::id()));
        std::fs::File::create(&path)
            .unwrap()
            .write_all(b"hello clipboard")
            .unwrap();

        let built = FileDescriptor::build(&path).unwrap();
        assert_eq!(built.len(), 592);

        let fd = FileDescriptor::parse(&built).unwrap();
        assert!(fd.name.contains("lamco_fd_roundtrip"));
        assert!(fd.name.ends_with(".txt"));
        assert_eq!(fd.size, Some(15)); // "hello clipboard"
        assert!(fd.flags.has_flag(FileDescriptorFlags::FILESIZE));

        std::fs::remove_file(&path).ok();
    }
}
