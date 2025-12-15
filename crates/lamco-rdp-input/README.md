# lamco-rdp-input

[![Crates.io](https://img.shields.io/crates/v/lamco-rdp-input.svg)](https://crates.io/crates/lamco-rdp-input)
[![Documentation](https://docs.rs/lamco-rdp-input/badge.svg)](https://docs.rs/lamco-rdp-input)
[![CI](https://github.com/lamco-admin/lamco-rdp/actions/workflows/ci.yml/badge.svg)](https://github.com/lamco-admin/lamco-rdp/actions)
[![License](https://img.shields.io/crates/l/lamco-rdp-input.svg)](LICENSE-MIT)

RDP input event translation for Rust - keyboard scancodes to evdev keycodes, mouse event handling, and multi-monitor coordinate transformation.

## Overview

This crate provides complete input event translation for RDP server implementations. It handles the conversion from RDP protocol input events to Linux evdev keycodes, enabling seamless integration with Wayland compositors and other Linux input systems.

The translation layer supports standard keyboards, extended multimedia keys, international layouts, and complex multi-monitor configurations with per-monitor DPI scaling.

## Features

- **Complete Keyboard Support**
  - 150+ scancode mappings (standard, extended E0, E1 prefix)
  - International layout support (US, DE, FR, UK, AZERTY, QWERTZ, Dvorak)
  - Full modifier tracking (Shift, Ctrl, Alt, Meta)
  - Toggle key handling (Caps Lock, Num Lock, Scroll Lock)
  - Key repeat detection with configurable timing

- **Advanced Mouse Support**
  - Absolute and relative movement
  - Sub-pixel precision with accumulation
  - 5-button support (Left, Right, Middle, Extra1, Extra2)
  - High-precision scrolling with accumulator
  - Button state tracking

- **Multi-Monitor Coordinate Transformation**
  - Complete transformation pipeline (RDP → Virtual Desktop → Monitor → Stream)
  - DPI scaling and monitor scale factor support
  - Mouse acceleration with Windows-style curves
  - Multi-monitor boundary handling

## Quick Start

```rust
use lamco_rdp_input::{InputTranslator, RdpInputEvent, MonitorInfo};

// Configure monitors
let monitors = vec![
    MonitorInfo {
        id: 1,
        name: "Primary".to_string(),
        x: 0, y: 0,
        width: 1920, height: 1080,
        dpi: 96.0,
        scale_factor: 1.0,
        stream_x: 0, stream_y: 0,
        stream_width: 1920, stream_height: 1080,
        is_primary: true,
    },
];

// Create translator
let mut translator = InputTranslator::new(monitors)?;

// Translate keyboard event
let event = RdpInputEvent::KeyboardScancode {
    scancode: 0x1E,  // 'A' key
    extended: false,
    e1_prefix: false,
    pressed: true,
};

let linux_event = translator.translate_event(event)?;
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
lamco-rdp-input = "0.1"
```

## Documentation

See [docs.rs/lamco-rdp-input](https://docs.rs/lamco-rdp-input) for full API documentation.

## About Lamco

This crate is part of the Lamco RDP project. Lamco develops RDP server solutions for Wayland/Linux.

**Open source foundation:** Protocol components, input translation, clipboard utilities
**Commercial products:** Lamco RDP Portal Server, Lamco VDI

Learn more: [lamco.ai](https://lamco.ai)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual licensed as above, without any additional terms or conditions.
