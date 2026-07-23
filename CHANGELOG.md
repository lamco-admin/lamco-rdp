# Changelog

All notable changes to the lamco-rdp workspace will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0] - 2026-07-23

### Changed
- Bumped bundled `lamco-rdp-clipboard` to 0.5.0 — `ironrdp-cliprdr` 0.6 → 0.7
  (transitively `ironrdp-pdu` 0.9 / `ironrdp-svc` 0.8) to track the current
  upstream IronRDP release. No source change; breaking for consumers of the
  re-exported clipboard types, which must move to `ironrdp-cliprdr` 0.7.

## [0.6.3] - 2026-06-30

### Changed
- Bumped bundled `lamco-rdp-clipboard` to 0.4.2 (adds `on_remote_file_list` →
  `ClipboardEvent::RemoteFileList` for eager clipboard file-list consumers).
- Refreshed dependencies; `bytes` ≥ 1.12.0 (RUSTSEC-2026-0007).

## [0.6.2] - 2026-06-03

### Changed
- Relicensed to Lamco Development LLC; removed `authors` metadata.
- Workspace MSRV raised to 1.89 to match the IronRDP cliprdr/core requirement.
- CI hardened: clippy `--all-targets` in both default and all-features modes,
  cargo-deny, a 1.89 MSRV gate, and a fuzz-smoke job. Added THIRD_PARTY_NOTICES.
- Dependency hygiene: dropped the unused `percent-encoding`; `bytes` and `image`
  now inherit their versions from the workspace.
- Sub-crate versions: lamco-clipboard-core 0.6.1, lamco-rdp-clipboard 0.4.1,
  lamco-rdp-input 0.1.5.

### Fixed
- lamco-clipboard-core 0.6.1: DIB image parsing no longer panics on oversized
  header dimensions (an integer-overflow guard, found by fuzzing). Added
  FILEDESCRIPTORW parser tests and clipboard fuzz targets.

## [0.6.1] - 2026-05-31

### Changed
- lamco-rdp-input 0.1.4: lock-key match-guard cleanup (clippy `collapsible_match`,
  no behavior change).

## [0.6.0] - 2026-05-31

### Changed
- **BREAKING:** lamco-rdp-clipboard 0.4.0 — `FileContentsRequest.index` is now `i32`
  (signed `lindex` per [MS-RDPECLIP] 2.2.5.3); requires `ironrdp-cliprdr` 0.6 and
  `ironrdp-core` 0.2.

## [0.5.3] - 2026-03-27

### Changed
- lamco-rdp-clipboard 0.3.0: RDP-specific types moved out of lamco-clipboard-core.
- Updated sub-crate dependency versions.

## [0.5.2] - 2026-03-15

### Changed
- Bump to Rust edition 2024, minimum supported Rust version 1.85
- Updated all sub-crate dependencies:
  - lamco-rdp-input 0.1.3
  - lamco-clipboard-core 0.5.2
  - lamco-rdp-clipboard 0.2.3

## [0.5.1] - 2026-01-06

### Changed
- Hash-based loop detection improvements in clipboard-core
- Expanded keyboard layouts in rdp-input

## [0.5.0] - 2025-12-30

### Changed
- Switched IronRDP deps from fork to upstream crates.io 0.5.0
- lamco-clipboard-core 0.5.0: DIBV5 format support
- lamco-rdp-clipboard 0.2.2: CB_FILECLIP_NO_FILE_PATHS capability

## [0.4.0] - 2025-12-24

### Changed
- lamco-clipboard-core 0.4.0: RTF and synthesized format support

## [0.3.0] - 2025-12-23

### Changed
- lamco-clipboard-core 0.3.0: FileGroupDescriptorW support
- lamco-rdp-clipboard 0.2.1: updated clipboard-core dep

## [0.2.0] - 2025-12-21

### Changed
- Updated lamco-clipboard-core to v0.2.0 (adds `image` feature)
- Updated lamco-rdp-clipboard to v0.2.0

## [0.1.1] - 2025-12-17

### Fixed

- Fixed docs.rs build failure by replacing deprecated `doc_auto_cfg` with `doc_cfg`
  - The `doc_auto_cfg` feature was removed in Rust 1.92.0 and merged into `doc_cfg`
- Updated `lamco-clipboard-core` dependency to 0.1.1
- Updated `lamco-rdp-input` dependency to 0.1.1

## [0.1.0] - 2025-12-16

### Added
- Initial workspace setup
- `lamco-rdp-input` crate: RDP input event translation
  - Keyboard scancode to evdev keycode translation (150+ mappings)
  - Extended E0 and E1 prefix handling
  - Multi-monitor coordinate transformation with DPI scaling
  - Mouse event handling with sub-pixel precision
  - International keyboard layout support
- `lamco-clipboard-core` crate: Protocol-agnostic clipboard utilities
  - `ClipboardSink` trait with 7 async methods (RPITIT)
  - `FormatConverter` for MIME ↔ Windows clipboard format conversion
  - `LoopDetector` with SHA256-based history and time-windowed detection
  - `TransferEngine` for chunked file transfers with integrity verification
- `lamco-rdp-clipboard` crate: IronRDP clipboard integration
  - `RdpCliprdrBackend` implementing IronRDP `CliprdrBackend` trait
  - Non-blocking event-based design for async processing
  - `RdpCliprdrFactory` for multiple RDP connections
  - `ClipboardEvent` enum for all CLIPRDR operations
