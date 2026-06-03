#![no_main]

use libfuzzer_sys::fuzz_target;
use lamco_clipboard_core::image;

// DIB / image header parsing is the richest integer-handling surface: width,
// height, bit-depth, and palette sizes come straight off the wire. These must
// not panic or attempt unbounded allocations on malformed input.
fuzz_target!(|data: &[u8]| {
    let _ = image::any_to_dib(data);
    let _ = image::any_to_dibv5(data);
    let _ = image::dib_dimensions(data);
    let _ = image::dib_to_png(data);
});
