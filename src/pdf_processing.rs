pub struct ExtractedText {
    pub full_text: String,
    pub first_page: String,
    pub page_count: usize,
}

pub fn extract_pdf(bytes: &[u8], maximum_bytes: usize) -> Result<ExtractedText, String> {
    if bytes.len() > maximum_bytes { return Err("PDF exceeds configured size limit".into()); }
    if !bytes.starts_with(b"%PDF-") { return Err("source is not a PDF file".into()); }
    let pages = pdf_extract::extract_text_from_mem_by_pages(bytes).map_err(|e| format!("PDF text extraction failed: {e}"))?;
    if pages.is_empty() { return Err("PDF contains no pages".into()); }
    let first_page = normalize_text(&pages[0]);
    let full_text = normalize_text(&pages.join("\n\n"));
    if !full_text.chars().any(char::is_alphanumeric) { return Err("PDF contains no usable native text".into()); }
    Ok(ExtractedText { full_text, first_page, page_count: pages.len() })
}

pub fn normalize_text(input: &str) -> String {
    let clean = input.replace('\0', "").replace('\u{fffd}', "").replace("\r\n", "\n").replace('\r', "\n");
    clean.lines().map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>().join("\n").trim().to_owned()
}

pub fn text_to_safe_html(text: &str) -> String {
    text.split("\n\n").map(str::trim).filter(|p| !p.is_empty()).map(|paragraph| {
        let escaped = paragraph.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
        format!("<p>{escaped}</p>")
    }).collect::<Vec<_>>().join("\n")
}

pub fn classify_promotion(filename: &str, first_page: &str) -> &'static str {
    let name = filename.to_lowercase().replace('ё', "е");
    let page = first_page.to_lowercase().replace('ё', "е");
    let explicit_signals = ["если вы еще не в клубе", "рекламный материал", "реклама", "join the club", "subscribe now"];
    if explicit_signals.iter().any(|signal| name.contains(signal) || page.contains(signal)) { "promotional" } else { "publish" }
}
