# lamco-rdp-input

RDP input event translation for Rust - keyboard scancodes to evdev keycodes, mouse event handling, and multi-monitor coordinate transformation.

[![Crates.io](https://img.shields.io/crates/v/lamco-rdp-input.svg)](https://crates.io/crates/lamco-rdp-input)
[![Documentation](https://docs.rs/lamco-rdp-input/badge.svg)](https://docs.rs/lamco-rdp-input)
[![License](https://img.shields.io/crates/l/lamco-rdp-input.svg)](LICENSE-MIT)

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

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
