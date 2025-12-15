# lamco-rdp

[![CI](https://github.com/lamco-admin/lamco-rdp/actions/workflows/ci.yml/badge.svg)](https://github.com/lamco-admin/lamco-rdp/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

RDP protocol implementations and IronRDP extensions for Rust.

## Overview

This workspace provides modular RDP protocol components for building RDP servers and clients in Rust. Each crate is designed to be independent and reusable, following the Unix philosophy of small, focused tools that do one thing well.

The crates integrate seamlessly with the [IronRDP](https://github.com/Devolutions/IronRDP) ecosystem while providing additional functionality for Wayland/Linux environments.

## Crates

| Crate | Version | Description |
|-------|---------|-------------|
| [lamco-rdp-input](crates/lamco-rdp-input) | [![Crates.io](https://img.shields.io/crates/v/lamco-rdp-input.svg)](https://crates.io/crates/lamco-rdp-input) | Input event translation (keyboard, mouse, coordinates) |
| [lamco-clipboard-core](crates/lamco-clipboard-core) | [![Crates.io](https://img.shields.io/crates/v/lamco-clipboard-core.svg)](https://crates.io/crates/lamco-clipboard-core) | Protocol-agnostic clipboard utilities |
| [lamco-rdp-clipboard](crates/lamco-rdp-clipboard) | [![Crates.io](https://img.shields.io/crates/v/lamco-rdp-clipboard.svg)](https://crates.io/crates/lamco-rdp-clipboard) | IronRDP clipboard integration |

## Quick Start

Add the meta-crate to your `Cargo.toml`:

```toml
[dependencies]
lamco-rdp = "0.1"
```

Or select specific features:

```toml
[dependencies]
lamco-rdp = { version = "0.1", default-features = false, features = ["input"] }
```

Or use individual crates directly:

```toml
[dependencies]
lamco-rdp-input = "0.1"
```

## Features

### Input Translation (`lamco-rdp-input`)

- Complete keyboard scancode → evdev keycode translation
- Multi-monitor coordinate transformation with DPI scaling
- Mouse event handling with sub-pixel precision
- International keyboard layout support

### Clipboard (`lamco-clipboard-core`, `lamco-rdp-clipboard`)

- Protocol-agnostic `ClipboardSink` trait
- Format conversion (MIME ↔ Windows clipboard formats)
- Loop detection for bidirectional sync
- File transfer support (MS-RDPECLIP FileContents)
- IronRDP `CliprdrBackend` implementation

## About Lamco

This workspace is part of the Lamco RDP project. Lamco develops RDP server solutions for Wayland/Linux.

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

Contributions are welcome! Please see our [GitHub repository](https://github.com/lamco-admin/lamco-rdp) for issues and pull requests.
