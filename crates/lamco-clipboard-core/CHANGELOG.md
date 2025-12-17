# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-12-17

### Fixed

- Fixed docs.rs build failure by replacing deprecated `doc_auto_cfg` with `doc_cfg`
  - The `doc_auto_cfg` feature was removed in Rust 1.92.0 and merged into `doc_cfg`
- Fixed code formatting issues in image module

## [0.1.0] - 2025-01-13

### Added

- Initial release
- **`ClipboardSink` trait** - Protocol-agnostic clipboard backend interface
  - 7 async methods: `announce_formats`, `read_clipboard`, `write_clipboard`, `subscribe_changes`, `get_file_list`, `read_file_chunk`, `write_file`
  - `FileInfo` struct for file transfer metadata
  - `ClipboardChange` notification struct
  - `ClipboardChangeReceiver` for change subscriptions
- **Format conversion** (`formats` module)
  - Windows clipboard format constants (CF_UNICODETEXT, CF_DIB, CF_HTML, etc.)
  - `ClipboardFormat` struct with ID and optional name
  - `mime_to_rdp_formats()` - Convert MIME types to RDP formats
  - `rdp_format_to_mime()` - Convert RDP format IDs to MIME types
  - `FormatConverter` for data conversion:
    - UTF-8 ↔ UTF-16LE (CF_UNICODETEXT)
    - HTML ↔ CF_HTML format
    - URI list ↔ HDROP format
- **Loop detection** (`loop_detector` module)
  - `LoopDetector` - Prevent clipboard sync loops
  - SHA256-based format and content hashing
  - Configurable time window (default: 500ms)
  - `ClipboardSource` enum (Rdp, Local)
- **Transfer engine** (`transfer` module)
  - `TransferEngine` - Chunked transfers for large data
  - Progress tracking with ETA calculation
  - SHA256 integrity verification
  - Configurable chunk size, max size, and timeout

[0.1.1]: https://github.com/lamco-admin/lamco-rdp/releases/tag/lamco-clipboard-core-v0.1.1
[0.1.0]: https://github.com/lamco-admin/lamco-rdp/releases/tag/lamco-clipboard-core-v0.1.0
