# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2026-07-23

### Changed
- **Bump `ironrdp-cliprdr` 0.6 → 0.7** (transitively `ironrdp-pdu` 0.9,
  `ironrdp-svc` 0.8, `ironrdp-core` 0.2.1), tracking the current upstream IronRDP
  release. No source change — the crate compiles unmodified against 0.7. The
  version bump is required because `ironrdp-cliprdr` types appear in this crate's
  public API (the `CliprdrBackend` impl and re-exported request/response types),
  so consumers must move to 0.7 in lockstep. This restores single-instance
  alignment for consumers (e.g. lamco-rdp-server) that `[patch]` `ironrdp-cliprdr`
  to a git fork now at 0.7, which otherwise pulled a second 0.6 copy and broke
  the `CliprdrBackend` trait match (the patch-diamond).

## [0.4.2] - 2026-06-30

### Added
- `CliprdrBackend::on_remote_file_list` is now implemented, surfacing the remote
  `FileGroupDescriptorW` file list (received in response to a paste request) as a
  new `ClipboardEvent::RemoteFileList { files, clip_data_id }`. Each file is an
  owned `RemoteFileMetadata` (name, size, relative path). This lets consumers
  pre-populate a local clipboard source with file URIs up front, which is required
  by eager/synchronous clipboards such as Wayland `ext-data-control` where data
  must be present before the compositor's `send` request rather than fetched on
  demand.

## [0.4.0] - 2026-05-31

### Changed
- **BREAKING:** `ClipboardEvent::FileContentsRequest.index` is now `i32` (was `u32`),
  matching the signed `lindex` field in `ironrdp-cliprdr` 0.6 per [MS-RDPECLIP] 2.2.5.3.
  Negative indices are rejected during decode.
- Updated `ironrdp-cliprdr` to 0.6 and `ironrdp-core` to 0.2.

## [0.3.0] - 2026-03-27

### Changed
- Moved RDP-specific clipboard types out of `lamco-clipboard-core` into this crate.

## [0.2.3] - 2026-03-15

### Changed
- Migrated to Rust edition 2024 (MSRV 1.85).

## [0.2.2] - 2025-12-24

### Added
- **CB_FILECLIP_NO_FILE_PATHS capability flag** for privacy
  - Prevents source file paths from being leaked in clipboard data
  - Enhances security for enterprise environments

### Changed
- Updated lamco-clipboard-core dependency to v0.4.0

## [0.2.1] - 2025-12-23

### Changed
- Updated lamco-clipboard-core dependency to v0.3.0 (adds FileGroupDescriptorW support)

## [0.2.0] - 2025-12-21

### Changed
- Updated lamco-clipboard-core dependency to v0.2.0

## [0.1.1] - 2025-12-17

### Fixed

- Fixed docs.rs build failure by replacing deprecated `doc_auto_cfg` with `doc_cfg`
  - The `doc_auto_cfg` feature was removed in Rust 1.92.0 and merged into `doc_cfg`

## [0.1.0] - 2025-01-13

### Added

- Initial release
- **`RdpCliprdrBackend`** - IronRDP `CliprdrBackend` implementation
  - Non-blocking event-based design for async processing
  - Supports all CLIPRDR operations: format list, data request/response, file transfer
  - Capability negotiation (long format names, file streaming, data locking)
- **`RdpCliprdrFactory`** - Factory for creating backend instances
  - Shared event channel across multiple RDP connections
  - Configurable temporary directory for file transfers
- **`ClipboardEvent`** enum for async event processing
  - Ready, RequestFormatList, NegotiatedCapabilities
  - RemoteCopy, FormatDataRequest, FormatDataResponse
  - FileContentsRequest, FileContentsResponse
  - Lock, Unlock
- **`ClipboardEventSender`** / **`ClipboardEventReceiver`** - Thread-safe event channel
- Re-exports of `lamco-clipboard-core` types for convenience
- Re-exports of commonly used IronRDP types

[0.2.0]: https://github.com/lamco-admin/lamco-rdp/releases/tag/lamco-rdp-clipboard-v0.2.0
[0.1.1]: https://github.com/lamco-admin/lamco-rdp/releases/tag/lamco-rdp-clipboard-v0.1.1
[0.1.0]: https://github.com/lamco-admin/lamco-rdp/releases/tag/lamco-rdp-clipboard-v0.1.0
