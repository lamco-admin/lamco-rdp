# Changelog

All notable changes to lamco-rdp-input will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2025-12-17

### Fixed

- Fixed docs.rs build failure by replacing deprecated `doc_auto_cfg` with `doc_cfg`
  - The `doc_auto_cfg` feature was removed in Rust 1.92.0 and merged into `doc_cfg`

## [0.1.0] - 2025-12-16

### Added
- Initial release extracted from wayland-rdp project
- `ScancodeMapper`: Keyboard scancode to evdev keycode translation
  - 150+ standard scancode mappings
  - Extended E0 prefix support (multimedia keys, navigation)
  - E1 prefix support (Pause/Break key)
  - International keyboard layout foundations
- `InputTranslator`: Unified input event translation
  - Keyboard event processing with modifier tracking
  - Mouse movement (absolute and relative)
  - Mouse button handling (5-button support)
  - High-precision scroll wheel with accumulator
- `CoordinateMapper`: Multi-monitor coordinate transformation
  - RDP coordinates to virtual desktop mapping
  - Virtual desktop to per-monitor local coordinates
  - DPI scaling and monitor scale factor support
  - Stream coordinate output for video pipeline
- `MonitorInfo`: Monitor configuration with complete metadata
- Error types with detailed context for debugging
