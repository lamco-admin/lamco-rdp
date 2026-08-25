# Changelog

All notable changes to lamco-rdp-input will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-08-25

### Added
- **MS-RDPEI multi-touch support** (`touch` module, new public
  `TouchHandler`/`TouchEvent`/`TouchContactFlags`). Tracks per-contact state
  (out-of-range / hovering / engaged) across up to 256 simultaneous contacts
  (`contactId` is a wire `u8`), turning each wire contact update into at
  most one host-facing `Down`/`Motion`/`Up` event. Hover-only motion
  produces no event (no hover primitive downstream); an illegal
  MS-RDPEI flag combination or a position outside all configured monitors
  is logged and suppressed rather than propagated as an error, so one bad
  contact in a multi-touch frame doesn't abort the rest of the frame.

### Changed (Breaking)
- **`CoordinateTransformer::rdp_to_stream` now takes `(i32, i32)`** instead
  of `(u32, u32)`. MS-RDPEI touch contacts carry signed
  `FOUR_BYTE_SIGNED_INTEGER` coordinates (can go negative for a monitor
  positioned left of or above the primary in an extended layout), unlike
  MS-RDPBCGR's unsigned mouse coordinates; both feed the same
  virtual-desktop-relative pixel space. Mouse callers widen their
  `u32`/`u16` values into the signed parameter.
- **`MouseHandler::handle_button_down`/`handle_button_up` gained two new
  required parameters**, `position: Option<(u32, u32)>` and
  `transformer: &mut CoordinateTransformer`, and **`MouseEvent::ButtonDown`/
  `ButtonUp` gained a new field**, `position: Option<(f64, f64)>`. Fixes
  [IronRDP#1466](https://github.com/Devolutions/IronRDP/issues/1466): a
  button PDU with no preceding Move (e.g. a touch/tap-style client such as
  iOS/iPadOS) previously landed at whatever the tracked cursor position
  happened to be, rather than where the client actually pressed. When
  `position` is `Some`, it is transformed and clamped the same way
  `handle_absolute_move` does, and updates the tracked cursor position
  *before* the button state changes. Pass `None` for sources with no
  absolute position to report (a relative-motion channel's button branch,
  `MouseEvent::ButtonRel`).

## [0.1.4] - 2026-05-31

### Changed
- Collapse CapsLock/NumLock/ScrollLock key handling into match guards
  (satisfies Rust 1.95 clippy `collapsible_match`). No behavior change.

## [0.1.3] - 2026-03-15

### Changed
- Bump to Rust edition 2024, minimum supported Rust version 1.85

## [0.1.2] - 2026-01-06

### Added
- Expanded keyboard layout support
- Hash-based loop detection improvements

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
