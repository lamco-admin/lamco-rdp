# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0] - 2026-10-30

### Changed
- **Moves to `ironrdp-cliprdr` 0.9** (with `ironrdp-pdu` 0.11 and `ironrdp-svc` 0.10),
  the IronRDP releases of 2026-10-27. No source change. `ironrdp-cliprdr` types appear
  in this crate's public API (the `CliprdrBackend` impl and the request and response
  types), so consumers move to 0.9 with it. A consumer that patches `ironrdp-cliprdr` to
  an IronRDP checkout at 0.9 (lamco-rdp-server does) otherwise pulls a second 0.8 copy
  from crates.io and the `CliprdrBackend` trait no longer matches. The minimum Rust
  version stays 1.94.

## [0.6.1] - 2026-10-24

### Fixed
- A file contents response with no data was reported as an error. The event
  took its error flag from the data being empty, so a successful read at the end
  of a file, which carries no data, failed. `ClipboardEvent::FileContentsResponse`
  now carries the response's own flag.

## [0.6.0] - 2026-10-24

### Changed
- **Moves to `ironrdp-cliprdr` 0.8 and `ironrdp-core` 0.3** (transitively
  `ironrdp-pdu` 0.10 and `ironrdp-svc` 0.9), the IronRDP releases of 2026-10-23.
  No source change. `ironrdp-cliprdr` types appear in this crate's public API
  (the `CliprdrBackend` impl and the request and response types), so consumers
  move to 0.8 with it. A consumer that patches `ironrdp-cliprdr` to an IronRDP
  checkout at 0.8 (lamco-rdp-server does) otherwise pulls a second 0.7 copy from
  crates.io and the `CliprdrBackend` trait no longer matches.
- **Minimum supported Rust version is now 1.94**, because `ironrdp-cliprdr` 0.8
  requires it. The other crates in the workspace stay at 1.89.

## [0.5.1] - 2026-09-30

### Added
- `ClipboardEventReceiver::recv()`: an async wait for the next event, so a
  consumer can sleep until the backend queues one instead of polling
  `try_recv()` on a timer. Runtime-agnostic; no new dependencies.

### Fixed
- `ClipboardEventSender::send()` dropped the event (with only a warning) when
  the queue lock was contended, for example while a consumer was polling. The
  queue now takes a plain mutex held for one push or pop, so no event is lost.

### Changed
- The UTF-16 decoding of file descriptors uses `as_chunks` instead of
  `chunks_exact`.

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
