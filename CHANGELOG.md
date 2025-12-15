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
- `lamco-clipboard-core` placeholder for protocol-agnostic clipboard utilities
- `lamco-rdp-clipboard` placeholder for IronRDP clipboard integration
