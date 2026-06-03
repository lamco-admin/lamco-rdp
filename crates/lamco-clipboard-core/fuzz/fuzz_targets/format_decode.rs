#![no_main]

use libfuzzer_sys::fuzz_target;
use lamco_clipboard_core::FormatConverter;

// Text decoders consume attacker-controlled CF_UNICODETEXT / CF_TEXT / CF_OEMTEXT
// payloads. Decoding must fail gracefully, never panic.
fuzz_target!(|data: &[u8]| {
    let conv = FormatConverter::new();
    let _ = conv.unicode_to_text(data);
    let _ = conv.ansi_to_text(data);
    let _ = conv.oem_to_text(data);
});
