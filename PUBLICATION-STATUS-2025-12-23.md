# lamco-rdp Publication Status - 2025-12-23

**Repository:** https://github.com/lamco-admin/lamco-rdp
**Date:** 2025-12-23
**Status:** All crates published ✅

---

## Published Crates

| Crate | Version | Published | crates.io |
|-------|---------|-----------|-----------|
| lamco-clipboard-core | v0.3.0 | 2025-12-23 | https://crates.io/crates/lamco-clipboard-core |
| lamco-rdp-clipboard | v0.2.1 | 2025-12-23 | https://crates.io/crates/lamco-rdp-clipboard |
| lamco-rdp-input | v0.1.1 | 2025-12-17 | https://crates.io/crates/lamco-rdp-input |
| lamco-rdp (meta) | v0.3.0 | 2025-12-23 | https://crates.io/crates/lamco-rdp |

---

## Changes in This Release

### lamco-clipboard-core v0.3.0 (NEW FEATURE)

**Files:**
- `src/formats.rs` (+309, -5 lines)
- `src/lib.rs` (+2, -1 lines)
- `src/sanitize.rs` (+554 lines, NEW FILE)

**New Features:**

1. **FileGroupDescriptorW Support** for RDP clipboard file transfer:
   - `FileDescriptor` struct for parsing 592-byte FILEDESCRIPTORW structures
   - `FileDescriptorFlags` for metadata field validation
   - `FileDescriptor::build()` to create descriptors from local files
   - `parse_list()` and `build_list()` for multiple file handling

2. **Format Constants:**
   - `CF_FILEGROUPDESCRIPTORW` = 49430
   - `CF_FILECONTENTS` = 49338

3. **Cross-platform Sanitization Module** (`sanitize.rs`):
   - Windows reserved name handling (CON, PRN, COM1-9, LPT1-9, AUX, NUL)
   - Invalid character filtering (\/:*?"<>|)
   - Trailing dots/spaces cleanup
   - Line ending conversion (LF ↔ CRLF)
   - Path component extraction and validation

4. **Updated Format Mapping:**
   - `mime_to_rdp_formats()` now advertises FileGroupDescriptorW for file URIs
   - `rdp_format_to_mime()` handles FileGroupDescriptorW format

**Impact:** Enables bidirectional RDP clipboard file transfer (Windows ↔ Linux)

**Testing:** ⚠️ Code complete, not yet integration-tested in server

---

### lamco-rdp-clipboard v0.2.1 (DEPENDENCY UPDATE)

**Files:**
- `Cargo.toml` (version bump)
- `CHANGELOG.md` (documentation)

**Changes:**
- Updated lamco-clipboard-core dependency: 0.2.0 → 0.3.0
- NO code changes

**Testing:** ✅ Works with IronRDP 0.4.x from crates.io

---

### lamco-rdp-input v0.1.1

**Status:** No changes (published previously)

---

### lamco-rdp v0.3.0 (META CRATE)

**Files:**
- `Cargo.toml` (version updates)

**Changes:**
- Version bump follows lamco-clipboard-core minor version
- Updated workspace dependencies

---

## IronRDP Dependency Management

### Fork Technique Used for Publication

**Challenge:** lamco-rdp-clipboard depends on IronRDP, but:
- Devolutions/IronRDP master bumped to v0.5.0 (unreleased)
- crates.io only has ironrdp-cliprdr v0.4.0

**Solution:** Use our fork (glamberson/IronRDP) which has version 0.4.0

**Configuration:**
```toml
ironrdp-cliprdr = { version = "0.4", git = "https://github.com/glamberson/IronRDP", branch = "master" }
ironrdp-core = { version = "0.1", git = "https://github.com/glamberson/IronRDP", branch = "master" }
```

**Why this works:**
- Our fork has version 0.4.0 matching crates.io
- lamco-rdp-clipboard v0.2.1 has NO code changes, works fine with 0.4
- Published crates use ironrdp-cliprdr 0.4 from crates.io (git spec stripped)
- Honest and accurate dependency declaration

**Documentation:** See `/home/greg/lamco-admin/projects/lamco-rdp/notes/IRONRDP-FORK-PUBLICATION-TECHNIQUE.md`

---

## IronRDP Contributions

We submitted PRs to Devolutions/IronRDP:
- #1063: fix(server): enable reqwest feature ✅ Merged
- #1064: feat(cliprdr): add clipboard data locking methods ✅ Passing CI
- #1065: feat(cliprdr): add request_file_contents method ✅ Passing CI
- #1066: feat(cliprdr): add SendFileContentsResponse message variant ✅ Passing CI

**When IronRDP publishes v0.5.0:**
- Server implementations can use our new file transfer methods
- We can switch back to Devolutions/IronRDP upstream

---

## Git Tags

- lamco-clipboard-core-v0.3.0
- lamco-rdp-clipboard-v0.2.1
- lamco-rdp-v0.3.0

---

## Commits (Clean - No Attribution)

```
6d53d80 fix: use our IronRDP fork for publication compatibility
4ed3f58 chore: bump lamco-rdp-clipboard and workspace for publication
73aae53 docs: update CHANGELOG for lamco-clipboard-core v0.3.0
1444005 chore: bump lamco-clipboard-core to v0.3.0 for publication
8d077e1 feat(clipboard-core): Add FileGroupDescriptorW support for RDP file transfer
```

**All authored by:** Greg Lamberson <greg@lamco.io> / lamco-office <office@lamco.io>

---

## Next Steps for Server Development

**wrd-server-specs can now:**

1. **Use published crates:**
   ```toml
   [dependencies]
   lamco-clipboard-core = "0.3.0"
   ```

2. **Access FileGroupDescriptorW support:**
   ```rust
   use lamco_clipboard_core::formats::{FileDescriptor, parse_list, build_list};
   ```

3. **Implement file I/O handlers** using the new infrastructure

**See handover document in wrd-server-specs for details.**
