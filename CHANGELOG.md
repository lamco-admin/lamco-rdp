# Changelog

All notable changes to the lamco-rdp workspace will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
