//! # lamco-rdp
//!
//! RDP protocol implementations and IronRDP extensions for Rust.
//!
//! This meta-crate provides convenient access to the lamco-rdp family of crates:
//!
//! - [`lamco_rdp_input`] - RDP input event translation (keyboard scancodes, mouse coordinates)
//! - [`lamco_clipboard_core`] - Protocol-agnostic clipboard utilities (format conversion, loop detection)
//! - [`lamco_rdp_clipboard`] - IronRDP clipboard integration
//!
//! ## Feature Flags
//!
//! - `input` (default) - Include input translation
//! - `clipboard-core` (default) - Include clipboard core utilities
//! - `clipboard-rdp` - Include IronRDP clipboard integration
//! - `full` - Enable all features
//!
//! ## Quick Start
//!
//! ```toml
//! [dependencies]
//! lamco-rdp = "0.1"
//! ```
//!
//! Or select specific features:
//!
//! ```toml
//! [dependencies]
//! lamco-rdp = { version = "0.1", default-features = false, features = ["input"] }
//! ```

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(feature = "input")]
pub use lamco_rdp_input as input;

#[cfg(feature = "clipboard-core")]
pub use lamco_clipboard_core as clipboard_core;

#[cfg(feature = "clipboard-rdp")]
pub use lamco_rdp_clipboard as clipboard_rdp;

/// Prelude module for convenient imports
pub mod prelude {
    #[cfg(feature = "input")]
    pub use lamco_rdp_input::{InputTranslator, KeyModifiers, LinuxInputEvent, MouseButton, RdpInputEvent};

    #[cfg(feature = "clipboard-core")]
    pub use lamco_clipboard_core::{ClipboardSink, FormatConverter, LoopDetector};
}
