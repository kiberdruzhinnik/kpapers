#![no_main]
#![allow(dead_code)]

#[path = "../../src/pdf_processing.rs"]
mod pdf_processing;
libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    const MAX_PDF_BYTES: usize = 2 * 1024 * 1024;
    let _ = pdf_processing::extract_pdf(data, MAX_PDF_BYTES);
});
