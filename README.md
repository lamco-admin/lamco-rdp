# lamco-rdp

RDP protocol implementations and IronRDP extensions for Rust.

[![Crates.io](https://img.shields.io/crates/v/lamco-rdp.svg)](https://crates.io/crates/lamco-rdp)
[![Documentation](https://docs.rs/lamco-rdp/badge.svg)](https://docs.rs/lamco-rdp)
[![License](https://img.shields.io/crates/l/lamco-rdp.svg)](LICENSE-MIT)

## Crates

| Crate | Description | Status |
|-------|-------------|--------|
| [lamco-rdp-input](crates/lamco-rdp-input) | Input event translation (keyboard, mouse, coordinates) | Ready |
| [lamco-clipboard-core](crates/lamco-clipboard-core) | Protocol-agnostic clipboard utilities | In Development |
| [lamco-rdp-clipboard](crates/lamco-rdp-clipboard) | IronRDP clipboard integration | In Development |

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
lamco-rdp = "0.1"
```

Or select specific features:

```toml
[dependencies]
lamco-rdp = { version = "0.1", default-features = false, features = ["input"] }
```

## Features

### Input Translation (`lamco-rdp-input`)

- Complete keyboard scancode → evdev keycode translation
- Multi-monitor coordinate transformation with DPI scaling
- Mouse event handling with sub-pixel precision
- International keyboard layout support

### Clipboard (Coming Soon)

- Protocol-agnostic `ClipboardSink` trait
- Format conversion (MIME ↔ Windows clipboard formats)
- Loop detection for bidirectional sync
- File transfer support (MS-RDPECLIP FileContents)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please see our [GitHub repository](https://github.com/lamco-admin/lamco-rdp) for issues and pull requests.
