#![no_main]

use libfuzzer_sys::fuzz_target;
use lamco_clipboard_core::sanitize;

// Clipboard file-URI lists and filenames arrive from the peer as untrusted
// bytes. None of these paths should panic, regardless of input.
fuzz_target!(|data: &[u8]| {
    let _ = sanitize::parse_file_uris(data);

    if let Ok(s) = std::str::from_utf8(data) {
        let _ = sanitize::parse_file_uri(s);
        let _ = sanitize::sanitize_filename_for_windows(s);
        let _ = sanitize::sanitize_filename_for_linux(s);
        let _ = sanitize::sanitize_text_for_windows(s);
        let _ = sanitize::sanitize_text_for_linux(s);
    }
});
