# Clipboard Architecture Decisions

**Date**: 2025-12-16
**Status**: Approved

## Overview

This document captures the architectural decisions for clipboard functionality across the lamco crate ecosystem. The goal is to provide reusable, well-layered public crates while keeping the final lamco-rdp-server product as thin glue code.

## Crate Responsibilities

### lamco-clipboard-core (Protocol-Agnostic Primitives)

**Purpose**: Core clipboard abstractions with no protocol-specific dependencies.

**Provides**:
- `ClipboardSink` trait (async interface for clipboard backends)
- `FormatConverter` (MIME ↔ Windows clipboard format conversion)
- `LoopDetector` (prevent clipboard sync loops)
- `TransferEngine` (chunked transfer for large data)
- Error types and common data structures

**Feature Flags**:
- `image` - Enable image format conversion (PNG, JPEG, BMP, DIB)
- `file-transfer` - Enable file transfer support (CF_HDROP, FileContents)

### lamco-rdp-clipboard (IronRDP Bridge)

**Purpose**: Bridge IronRDP's CLIPRDR static virtual channel to ClipboardSink.

**Provides**:
- `RdpCliprdrBackend` implementing IronRDP's `CliprdrBackend` trait
- `RdpCliprdrFactory` for multi-connection support
- `ClipboardEvent` enum and non-blocking event queue
- Re-exports from lamco-clipboard-core for convenience

**Dependencies**: ironrdp-cliprdr (via git rev until 0.5 published)

### lamco-portal (XDG Desktop Portal Integration)

**Purpose**: Portal-based clipboard and screen capture for Wayland.

**Clipboard Components**:
- `ClipboardManager` - Low-level Portal Clipboard D-Bus API wrapper
- `PortalClipboardSink` - ClipboardSink trait implementation for Portal

**Feature Flags**:
- `clipboard-sink` - Enable ClipboardSink implementation (requires lamco-clipboard-core)
- `dbus-clipboard` - Enable D-Bus clipboard bridge for GNOME fallback

### lamco-rdp-server (Product - Thin Glue)

**Purpose**: Final RDP server product. Wires published crates together.

**Clipboard Role**: Minimal glue code that:
- Creates Portal session with clipboard enabled
- Instantiates PortalClipboardSink and RdpCliprdrBackend
- Wires events between them
- Handles server-specific policy (timeouts, error recovery)

## Architectural Decisions

### Decision 1: Image Conversion in lamco-clipboard-core

**Choice**: Add image format conversion to lamco-clipboard-core behind `image` feature flag.

**Rationale**:
- High reuse value across different clipboard implementations
- Keeps core crate optional-dependency-light when feature disabled
- Windows clipboard heavily uses DIB format; conversion is essential for images

**Formats Supported**:
- PNG ↔ DIB (Device Independent Bitmap)
- JPEG ↔ DIB
- BMP ↔ DIB
- GIF (read-only, converts to PNG for output)

**Implementation**: Use `image` crate for decoding/encoding.

### Decision 2: D-Bus Clipboard Bridge in lamco-portal

**Choice**: Add D-Bus clipboard bridge to lamco-portal as optional `dbus-clipboard` feature.

**Rationale**:
- GNOME's Portal implementation doesn't reliably emit SelectionOwnerChanged
- The wayland-rdp-clipboard GNOME Shell extension provides a workaround via D-Bus
- Keeping it in lamco-portal (not server) allows reuse by other Portal-based tools

**D-Bus Interface**:
```
Service: org.wayland_rdp.Clipboard
Path: /org/wayland_rdp/Clipboard
Interface: org.wayland_rdp.Clipboard
Signal: ClipboardChanged(mime_types: Vec<String>, content_hash: String)
```

### Decision 3: IronRDP Dependency Management

**Choice**: Use git revision in Cargo.toml until IronRDP publishes new crate version.

**Rationale**:
- Our code is now synchronized with IronRDP main branch
- IronRDP doesn't publish frequently; waiting could delay progress
- Git rev provides stability (pinned to specific commit)

**Cargo.toml Pattern**:
```toml
[workspace.dependencies]
ironrdp-cliprdr = { git = "https://github.com/Devolutions/IronRDP", branch = "master" }
ironrdp-core = { git = "https://github.com/Devolutions/IronRDP", branch = "master" }
```

**Current Reference**: Branch `master`, resolves to commit `b50b6483` as of 2025-12-16.

**Transition Plan**: Switch to crates.io version when IronRDP 0.5+ publishes.

### Decision 4: Rate Limiting in LoopDetector

**Choice**: Add rate limiting to lamco-clipboard-core's LoopDetector as configurable option.

**Rationale**:
- Belt-and-suspenders defense against rapid clipboard sync storms
- Content hashing alone may miss edge cases (same content, different ownership)
- Configurable allows server to enable, library users to disable if unwanted

**Implementation**:
```rust
pub struct LoopDetectionConfig {
    pub window_ms: u64,           // Time window for loop detection (default: 500ms)
    pub max_history: usize,       // Max operations to track (default: 10)
    pub enable_content_hashing: bool,  // SHA256 content dedup (default: true)
    pub rate_limit_ms: Option<u64>,    // Optional rate limit (default: None)
}
```

### Decision 5: Server as Thin Glue (Hybrid Refactor)

**Choice**: Refactor lamco-rdp-server to use published crates, keeping only glue code.

**Current State**: ~5,700 LOC monolithic clipboard implementation duplicating published crate functionality.

**Target State**: ~500-800 LOC glue code that:
- Initializes Portal session with clipboard
- Creates PortalClipboardSink and RdpCliprdrBackend instances
- Routes events between them
- Applies server-specific policy
- Handles D-Bus bridge for GNOME fallback

**Code Reduction**: ~80% reduction in server clipboard code.

## Data Flow

### Windows → Linux (Paste)

```
RDP Client copies
    ↓
IronRDP CLIPRDR FormatList PDU
    ↓
RdpCliprdrBackend.on_remote_copy() → ClipboardEvent::RemoteCopy
    ↓
Server glue receives event
    ↓
PortalClipboardSink.announce_formats(mime_types)
    ↓
Portal SetSelection (announces available formats)
    ↓
User pastes in Linux app
    ↓
Portal SelectionTransfer signal (with serial)
    ↓
PortalClipboardSink transfer listener
    ↓
Provides queued data via SelectionWrite
```

### Linux → Windows (Copy)

```
Wayland clipboard change
    ↓
Portal SelectionOwnerChanged signal (or D-Bus fallback)
    ↓
PortalClipboardSink.subscribe_changes() notification
    ↓
Server glue receives ClipboardChange
    ↓
LoopDetector.would_cause_loop() check
    ↓
If not loop: FormatConverter.mime_to_rdp_formats()
    ↓
Send to RDP via IronRDP message proxy
```

## File Structure After Refactor

```
lamco-clipboard-core/
├── src/
│   ├── lib.rs
│   ├── error.rs
│   ├── sink.rs          # ClipboardSink trait
│   ├── formats.rs       # FormatConverter + constants
│   ├── transfer.rs      # TransferEngine
│   ├── loop_detector.rs # LoopDetector with rate limiting
│   └── image.rs         # Image conversion (feature-gated)

lamco-rdp-clipboard/
├── src/
│   ├── lib.rs
│   ├── backend.rs       # RdpCliprdrBackend
│   ├── factory.rs       # RdpCliprdrFactory
│   ├── event.rs         # ClipboardEvent system
│   └── error.rs

lamco-portal/
├── src/
│   ├── clipboard.rs         # ClipboardManager
│   ├── clipboard_sink.rs    # PortalClipboardSink
│   └── dbus_clipboard.rs    # D-Bus bridge (feature-gated)

lamco-rdp-server/
├── src/
│   └── clipboard/
│       └── mod.rs       # Thin glue (~500-800 LOC)
```

## Implementation Order

1. Add `image` feature to lamco-clipboard-core
2. Enhance LoopDetector with rate limiting
3. Add D-Bus clipboard bridge to lamco-portal
4. Update lamco-rdp-clipboard to use IronRDP git rev
5. Refactor lamco-rdp-server to use published crates
6. Test end-to-end clipboard functionality
7. Publish updated crate versions

## Version Compatibility

| Crate | Current | After Refactor |
|-------|---------|----------------|
| lamco-clipboard-core | 0.1.0 | 0.2.0 |
| lamco-rdp-clipboard | 0.1.0 | 0.2.0 |
| lamco-portal | 0.1.1 | 0.2.0 |
| lamco-rdp-server | N/A | 0.1.0 |
