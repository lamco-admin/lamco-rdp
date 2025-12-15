//! # lamco-clipboard-core
//!
//! Protocol-agnostic clipboard utilities for Rust.
//!
//! This crate provides core clipboard functionality that can be used with any
//! clipboard backend (Portal, X11, headless, etc.):
//!
//! - **Format conversion** - MIME ↔ Windows clipboard format conversion
//! - **Loop detection** - Prevent clipboard sync loops with content hashing
//! - **Transfer engine** - Chunked transfer for large clipboard data
//! - **ClipboardSink trait** - Abstract clipboard backend interface
//!
//! ## Status
//!
//! This crate is under development. See [GitHub issue #1](https://github.com/lamco-admin/lamco-rdp/issues/1)
//! for the ClipboardSink trait design.

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

// Placeholder - full implementation coming soon
pub use std::result::Result;

/// Placeholder for ClipboardSink trait
pub trait ClipboardSink: Send + Sync {
    // Trait definition will be implemented per issue #1
}

/// Placeholder for FormatConverter
pub struct FormatConverter;

/// Placeholder for LoopDetector
pub struct LoopDetector;
