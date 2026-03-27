//! # lamco-clipboard-core
//!
//! Clipboard format conversion and synchronization utilities for Rust.
//!
//! This crate provides core clipboard infrastructure that works independently
//! of any specific remote desktop protocol:
//!
//! - **[`FormatConverter`]** - MIME type and Windows clipboard format conversion
//! - **[`LoopDetector`]** - Prevent clipboard sync loops with content hashing
//! - **[`TransferEngine`]** - Chunked transfer for large clipboard data
//! - **[`sanitize`]** - Cross-platform filename sanitization and file URI parsing
//!
//! ## Quick Start
//!
//! ```rust
//! use lamco_clipboard_core::{FormatConverter, LoopDetector};
//! use lamco_clipboard_core::formats::{ClipboardFormat, mime_to_rdp_formats};
//!
//! // Convert MIME types to clipboard formats
//! let formats = mime_to_rdp_formats(&["text/plain", "text/html"]);
//!
//! // Check for clipboard loops
//! let mut detector = LoopDetector::new();
//! if !detector.would_cause_loop(&formats) {
//!     // Safe to sync
//! }
//! ```
//!
//! ## Feature Flags
//!
//! - `image` - Enable image format conversion (PNG, JPEG, BMP to Windows DIB)
//!
//! ## Architecture
//!
//! This crate handles format conversion, loop detection, and transfer mechanics.
//! For the clipboard backend trait (`ClipboardSink`) and RDP-specific file
//! transfer types, see [`lamco-rdp-clipboard`](https://crates.io/crates/lamco-rdp-clipboard).

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]

mod error;
mod transfer;

pub mod formats;
pub mod loop_detector;
pub mod sanitize;

#[cfg(feature = "image")]
pub mod image;

pub use error::{ClipboardError, ClipboardResult};
pub use formats::{ClipboardFormat, FormatConverter};
pub use loop_detector::{ClipboardSource, LoopDetectionConfig, LoopDetector};
pub use transfer::{
    DEFAULT_CHUNK_SIZE, DEFAULT_MAX_SIZE, DEFAULT_TIMEOUT_MS, TransferConfig, TransferEngine, TransferProgress,
    TransferState,
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::formats::{mime_to_rdp_formats, rdp_format_to_mime};
    pub use crate::{ClipboardError, ClipboardResult, FormatConverter, LoopDetector};
}
