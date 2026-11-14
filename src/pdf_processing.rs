use pdf_oxide::PdfDocument;
use pdf_oxide::layout::TextLine;

#[derive(Debug)]
pub struct ExtractedText {
    pub full_text: String,
    pub html: String,
    pub first_page: String,
    pub page_count: usize,
}

const EXTRACTION_PIPELINE_REVISION: u32 = 15;

#[derive(Clone, Debug)]
struct StyledRun {
    text: String,
    bold: bool,
    italic: bool,
}

#[derive(Clone, Debug)]
struct StyledLine {
    runs: Vec<StyledRun>,
    size: f32,
    gap_before: f32,
    left: f32,
}

impl StyledLine {
    fn text(&self) -> String {
        self.runs.iter().map(|run| run.text.as_str()).collect()
    }
}

pub fn cache_fingerprint(source_fingerprint: &str) -> String {
    format!("pdf-text-v{EXTRACTION_PIPELINE_REVISION}:pdf_oxide:{source_fingerprint}")
}

pub fn extract_pdf(bytes: &[u8], maximum_bytes: usize) -> Result<ExtractedText, String> {
    if bytes.len() > maximum_bytes {
        return Err("PDF exceeds configured size limit".into());
    }
    if !bytes.starts_with(b"%PDF-") {
        return Err("source is not a PDF file".into());
    }

    let document = PdfDocument::from_bytes(bytes.to_vec())
        .map_err(|error| format!("PDF document load failed: {error}"))?;
    let page_count = document
        .page_count()
        .map_err(|error| format!("PDF page count failed: {error}"))?;
    if page_count == 0 {
        return Err("PDF contains no pages".into());
    }

    let mut pages = Vec::with_capacity(page_count);
    for page_index in 0..page_count {
        let lines = document.extract_text_lines(page_index).map_err(|error| {
            format!(
                "PDF text extraction failed on page {}: {error}",
                page_index + 1
            )
        })?;
        pages.push(text_lines_to_styled_lines(lines));
    }

    finalize_pages(pages, page_count)
}

fn text_lines_to_styled_lines(lines: Vec<TextLine>) -> Vec<StyledLine> {
    let mut styled = Vec::with_capacity(lines.len());
    let mut previous_top = None;
    for line in lines {
        let mut runs = Vec::new();
        for word in &line.words {
            let text = word.text.trim();
            if text.is_empty() {
                continue;
            }
            let mut run_text = String::new();
            if !runs.is_empty() {
                run_text.push(' ');
            }
            run_text.push_str(text);
            runs.push(StyledRun {
                text: run_text,
                bold: word.is_bold,
                italic: word.is_italic,
            });
        }
        if runs.is_empty() {
            continue;
        }
        let size = line
            .words
            .iter()
            .map(|word| word.avg_font_size)
            .fold(0.0_f32, f32::max)
            .max(1.0);
        let gap_before = previous_top
            .map(|top: f32| (top - line.bbox.y).max(0.0))
            .unwrap_or(0.0);
        previous_top = Some(line.bbox.y);
        styled.push(StyledLine {
            runs,
            size,
            gap_before,
            left: line.bbox.x,
        });
    }
    styled
}

fn finalize_pages(
    mut pages: Vec<Vec<StyledLine>>,
    page_count: usize,
) -> Result<ExtractedText, String> {
    if pages
        .last()
        .is_some_and(|lines| is_skladchik_boundary_ad(lines))
    {
        pages.pop();
    }
    if pages
        .first()
        .is_some_and(|lines| is_skladchik_boundary_ad(lines))
    {
        pages.remove(0);
    }
    // Printed page numbering normally starts at 1 on the first content page, so
    // labels must be compared after leading advertisement pages are removed.
    strip_page_labels_by_content_position(&mut pages);
    if let Some(first_page) = pages.first_mut() {
        strip_sponsr_preamble(first_page);
    }
    strip_sponsr_branding(&mut pages);

    let first_page = pages
        .first()
        .map(|lines| {
            normalize_text(
                &lines
                    .iter()
                    .map(StyledLine::text)
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
        })
        .unwrap_or_default();
    let page_text = pages_to_text(&pages);
    let full_text = normalize_text(&page_text);
    if !full_text.chars().any(char::is_alphanumeric) {
        return Err("PDF contains no usable native text".into());
    }
    let html = lines_to_html(&pages);
    Ok(ExtractedText {
        full_text,
        html,
        first_page,
        page_count,
    })
}

pub fn repair_sponsr_title_from_filename(filename: &str, extracted: &mut ExtractedText) {
    let Some(stem) = filename
        .strip_suffix(".pdf")
        .or_else(|| filename.strip_suffix(".PDF"))
    else {
        return;
    };
    let stem = stem
        .strip_suffix(" [skladchik.org]")
        .or_else(|| stem.strip_suffix(" [sklad"))
        .unwrap_or(stem);
    let title = stem
        .find('[')
        .map(|index| &stem[index..])
        .unwrap_or(stem)
        .replace("FAQ_", "FAQ:");
    if !title.starts_with('[') {
        return;
    }

    let mut blocks = extracted.html.lines().collect::<Vec<_>>();
    while blocks.first().is_some_and(|block| {
        let text = strip_html_tags(block).to_lowercase().replace('ё', "е");
        text == "sponsr"
            || text.contains("crimsonalter")
            || has_publication_timestamp(&text)
            || ["минута", "минут", "часа", "часов"]
                .iter()
                .any(|unit| text.contains(unit))
    }) {
        blocks.remove(0);
    }
    if blocks.is_empty() {
        blocks = vec![];
    }
    let recovered_title_block = blocks.first().is_some_and(|block| {
        strip_html_tags(block)
            .trim_start()
            .starts_with("[Философское]")
    });
    if recovered_title_block {
        blocks.remove(0);
    }
    let mut html = format!("<h1><strong>{}</strong></h1>", escape_html(&title));
    let mut inside_quote = false;
    for block in blocks {
        html.push('\n');
        inside_quote |= block.contains("<blockquote>");
        // Filename-recovered Sponsr titles are authoritative. Keep only short,
        // explicitly bold section headings; font-size-only candidates become
        // paragraphs, and notice contents stay paragraphs inside the quote.
        let block_text = strip_html_tags(block);
        let block = if inside_quote {
            block
                .replace("<h1>", "<p>")
                .replace("</h1>", "</p>")
                .replace("<h2>", "<p>")
                .replace("</h2>", "</p>")
                .replace("<h3>", "<p>")
                .replace("</h3>", "</p>")
        } else if (block.starts_with("<h2>") || block.starts_with("<h3>"))
            && block.contains("<strong>")
            && block_text.trim() != "это не инвестиционные рекомендации"
            && block_text.chars().count() <= 100
        {
            block.to_owned()
        } else {
            block
                .strip_prefix("<h1>")
                .and_then(|rest| rest.strip_suffix("</h1>"))
                .or_else(|| {
                    block
                        .strip_prefix("<h2>")
                        .and_then(|rest| rest.strip_suffix("</h2>"))
                })
                .or_else(|| {
                    block
                        .strip_prefix("<h3>")
                        .and_then(|rest| rest.strip_suffix("</h3>"))
                })
                .map(|body| format!("<p>{body}</p>"))
                .unwrap_or_else(|| block.to_owned())
        };
        html.push_str(&block);
        if block.contains("</blockquote>") {
            inside_quote = false;
        }
    }
    extracted.html = html;
    extracted.first_page = format!(
        "{title}\n{}",
        remove_publication_timestamp(&extracted.first_page)
    );
    extracted.full_text = format!(
        "{title}\n\n{}",
        remove_publication_timestamp(&extracted.full_text)
    );
}

fn strip_html_tags(text: &str) -> String {
    let mut inside_tag = false;
    text.chars()
        .filter(|ch| match *ch {
            '<' => {
                inside_tag = true;
                false
            }
            '>' => {
                inside_tag = false;
                false
            }
            _ if inside_tag => false,
            _ => true,
        })
        .collect::<String>()
        .trim()
        .to_owned()
}

fn has_publication_timestamp(text: &str) -> bool {
    let lower = text.to_lowercase();
    let has_month = [
        "янв", "фев", "мар", "апр", "май", "июн", "июл", "авг", "сен", "окт", "ноя", "дек",
    ]
    .iter()
    .any(|month| lower.contains(month));
    has_month && text.split_whitespace().any(|part| {
        let mut split = part.split(':');
        matches!((split.next(), split.next()), (Some(hour), Some(minute))
            if hour.len() == 2 && minute.len() == 2 && hour.chars().all(|ch| ch.is_ascii_digit()) && minute.chars().all(|ch| ch.is_ascii_digit()))
    })
}

fn remove_publication_timestamp(text: &str) -> String {
    let Some(line_end) = text.find('\n') else {
        let end = ["минута", "минут", "час", "часа", "часов"]
            .iter()
            .filter_map(|unit| text.find(unit).map(|index| index + unit.len()))
            .max();
        return end.map_or_else(
            || text.to_owned(),
            |end| {
                text[end..]
                    .trim_start_matches(|ch: char| ch.is_whitespace() || ch == '�')
                    .to_owned()
            },
        );
    };
    let first_line = &text[..line_end];
    let rest = &text[line_end + 1..];
    if has_publication_timestamp(first_line) {
        return rest.trim_start().to_owned();
    }
    text.to_owned()
}

fn push_styled_char(runs: &mut Vec<StyledRun>, ch: char, bold: bool, italic: bool) {
    if let Some(run) = runs
        .last_mut()
        .filter(|run| run.bold == bold && run.italic == italic)
    {
        run.text.push(ch);
    } else {
        runs.push(StyledRun {
            text: ch.to_string(),
            bold,
            italic,
        });
    }
}

fn strip_page_label_lines(lines: &mut Vec<StyledLine>, page_number: usize) {
    let expected = page_number.to_string();
    lines.retain(|line| line.text().trim() != expected);
}

fn strip_page_labels_by_content_position(pages: &mut [Vec<StyledLine>]) {
    for (page_index, lines) in pages.iter_mut().enumerate() {
        strip_page_label_lines(lines, page_index + 1);
    }
}

fn is_skladchik_promotion(text: &str) -> bool {
    let normalized = normalized_promotion_text(text);
    normalized.contains("skladchik org")
        && normalized.contains("ты еще не в клубе")
        && normalized.contains("что такое клуб складчик")
}

fn is_skladchik_purchase_promotion(text: &str) -> bool {
    let normalized = normalized_promotion_text(text);
    normalized.starts_with("куплено на") && normalized.contains("skladchik org")
}

fn normalized_promotion_text(text: &str) -> String {
    text.to_lowercase()
        .replace('ё', "е")
        .chars()
        .map(|ch| if ch.is_alphanumeric() { ch } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_skladchik_boundary_ad(lines: &[StyledLine]) -> bool {
    let text = lines
        .iter()
        .map(StyledLine::text)
        .collect::<Vec<_>>()
        .join("\n");
    is_skladchik_promotion(&text) || is_skladchik_purchase_promotion(&text)
}

#[cfg(test)]
fn is_xml_10_char(ch: char) -> bool {
    matches!(ch, '\u{9}' | '\u{a}' | '\u{d}')
        || ('\u{20}'..='\u{d7ff}').contains(&ch)
        || ('\u{e000}'..='\u{fffd}').contains(&ch)
        || ('\u{10000}'..='\u{10ffff}').contains(&ch)
}

pub fn normalize_text(input: &str) -> String {
    let clean = input
        .replace(['\0', '\u{fffd}'], "")
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    let mut paragraphs = Vec::new();
    let mut current = Vec::new();
    for line in clean.lines() {
        let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
        if line.is_empty() {
            if !current.is_empty() {
                paragraphs.push(current.join(" "));
                current.clear();
            }
        } else {
            current.push(line);
        }
    }
    if !current.is_empty() {
        paragraphs.push(current.join(" "));
    }
    paragraphs.join("\n\n")
}

#[cfg(test)]
pub fn text_to_safe_html(text: &str) -> String {
    text.split("\n\n")
        .map(str::trim)
        .filter(|paragraph| !paragraph.is_empty())
        .map(|paragraph| format!("<p>{}</p>", escape_html(paragraph)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn styled_runs_html(runs: &[StyledRun]) -> String {
    let mut coalesced = Vec::new();
    let mut index = 0;
    while index < runs.len() {
        let run = &runs[index];
        if run.text.chars().all(char::is_whitespace)
            && !coalesced.is_empty()
            && index + 1 < runs.len()
            && coalesced.last().is_some_and(|last: &StyledRun| {
                last.bold == runs[index + 1].bold && last.italic == runs[index + 1].italic
            })
        {
            coalesced
                .last_mut()
                .expect("checked above")
                .text
                .push_str(&run.text);
            coalesced
                .last_mut()
                .expect("checked above")
                .text
                .push_str(&runs[index + 1].text);
            index += 2;
            continue;
        }
        append_runs(&mut coalesced, std::slice::from_ref(run));
        index += 1;
    }

    let mut html = String::new();
    let mut open_tags: Vec<&str> = Vec::new();
    for run in &coalesced {
        // Keep italic as the outer element so bold phrases inside an italic
        // passage do not split the passage into repeated <em> elements.
        let mut tags = Vec::new();
        if run.italic {
            tags.push("em");
        }
        if run.bold {
            tags.push("strong");
        }

        let shared = open_tags
            .iter()
            .zip(&tags)
            .take_while(|(open, wanted)| open == wanted)
            .count();
        for tag in open_tags[shared..].iter().rev() {
            html.push_str(&format!("</{tag}>"));
        }
        for tag in &tags[shared..] {
            html.push_str(&format!("<{tag}>"));
        }
        open_tags = tags;
        html.push_str(&escape_html(&run.text));
    }
    for tag in open_tags.iter().rev() {
        html.push_str(&format!("</{tag}>"));
    }
    html
}

fn append_runs(target: &mut Vec<StyledRun>, runs: &[StyledRun]) {
    for run in runs {
        if let Some(last) = target
            .last_mut()
            .filter(|last| last.bold == run.bold && last.italic == run.italic)
        {
            last.text.push_str(&run.text);
        } else {
            target.push(run.clone());
        }
    }
}

fn first_content_style(line: &StyledLine) -> Option<(bool, bool)> {
    line.runs
        .iter()
        .find(|run| !run.text.trim().is_empty())
        .map(|run| (run.bold, run.italic))
}

fn last_content_style(line: &StyledLine) -> Option<(bool, bool)> {
    line.runs
        .iter()
        .rev()
        .find(|run| !run.text.trim().is_empty())
        .map(|run| (run.bold, run.italic))
}

fn push_line_separator(target: &mut Vec<StyledRun>, next_runs: &[StyledRun]) {
    let Some(previous) = target.last() else {
        return;
    };
    let next = next_runs.first();
    let previous_has_space = previous
        .text
        .chars()
        .last()
        .is_some_and(char::is_whitespace);
    let next_has_space = next
        .and_then(|run| run.text.chars().next())
        .is_some_and(char::is_whitespace);
    if previous_has_space || next_has_space {
        return;
    }
    let bold = next.is_some_and(|next| previous.bold && next.bold);
    let italic = next.is_some_and(|next| previous.italic && next.italic);
    push_styled_char(target, ' ', bold, italic);
}

fn body_text_size(pages: &[Vec<StyledLine>]) -> f32 {
    let mut sizes = pages
        .iter()
        .flatten()
        .filter(|line| !line.text().trim().is_empty())
        .map(|line| line.size)
        .collect::<Vec<_>>();
    sizes.sort_by(f32::total_cmp);
    let mut size_counts = Vec::<(f32, usize)>::new();
    for size in sizes {
        if let Some((_, count)) = size_counts
            .iter_mut()
            .find(|(known, _)| (size - *known).abs() < 0.5)
        {
            *count += 1;
        } else {
            size_counts.push((size, 1));
        }
    }
    size_counts
        .iter()
        .max_by(|(left_size, left_count), (right_size, right_count)| {
            left_count
                .cmp(right_count)
                .then_with(|| right_size.total_cmp(left_size))
        })
        .map(|(size, _)| *size)
        .unwrap_or(12.0)
        .max(1.0)
}

fn is_heading_candidate(line: &StyledLine, body_size: f32) -> bool {
    let text_length = line.text().chars().count();
    if text_length > 160 {
        return false;
    }

    let content_runs = line
        .runs
        .iter()
        .filter(|run| run.text.chars().any(|ch| !ch.is_whitespace()))
        .collect::<Vec<_>>();
    let fully_bold = !content_runs.is_empty() && content_runs.iter().all(|run| run.bold);
    let short_colon_heading = text_length <= 80 && line.text().trim_end().ends_with(':');
    let emphasized_heading = line.size >= body_size * 1.12 && (fully_bold || short_colon_heading);
    let prominent_and_isolated =
        line.size >= body_size * 1.35 && line.gap_before >= body_size * 1.5 && text_length <= 80;

    emphasized_heading || prominent_and_isolated
}

fn page_boundary_continues(previous: &StyledLine, next: &StyledLine, body_size: f32) -> bool {
    let previous_text = previous.text();
    let next_text = next.text();
    let previous_end = previous_text.trim_end_matches(|ch: char| {
        ch.is_whitespace() || matches!(ch, '"' | '\'' | '»' | '”' | '’' | ')')
    });
    let Some(last_char) = previous_end.chars().next_back() else {
        return false;
    };
    if matches!(last_char, '.' | '!' | '?' | '…' | ':' | ';') {
        return false;
    }
    let Some(first_char) = next_text.trim_start().chars().next() else {
        return false;
    };
    if !first_char.is_lowercase()
        || is_heading_candidate(previous, body_size)
        || is_heading_candidate(next, body_size)
    {
        return false;
    }

    let similar_size = (previous.size - next.size).abs() <= previous.size.max(next.size) * 0.05;
    let matching_style = last_content_style(previous)
        .zip(first_content_style(next))
        .is_some_and(|(before, after)| before == after);
    let compatible_position = (previous.left - next.left).abs() <= previous.size * 0.5;
    similar_size && matching_style && compatible_position
}

fn large_gap_continues_sentence(previous: &StyledLine, next: &StyledLine) -> bool {
    let previous_text = previous.text();
    let next_text = next.text();
    let previous_end = previous_text.trim_end();
    let Some(first_char) = next_text.trim_start().chars().next() else {
        return false;
    };

    let continues_after_punctuation = previous_end
        .chars()
        .next_back()
        .is_some_and(|last_char| matches!(last_char, ',' | ';' | ':'))
        && first_char.is_lowercase();
    let continues_after_honorific = previous_end
        .to_lowercase()
        .strip_suffix("ув.")
        .is_some_and(|prefix| prefix.chars().next_back().is_none_or(char::is_whitespace))
        && first_char.is_uppercase();
    if !continues_after_punctuation && !continues_after_honorific {
        return false;
    }

    let similar_size = (previous.size - next.size).abs() <= previous.size.max(next.size) * 0.05;
    let matching_style = last_content_style(previous)
        .zip(first_content_style(next))
        .is_some_and(|(before, after)| before == after);
    let compatible_position = (previous.left - next.left).abs() <= previous.size * 0.5;
    similar_size && matching_style && compatible_position
}

fn large_gap_starts_paragraph(previous: &StyledLine, next: &StyledLine) -> bool {
    next.gap_before > next.size.max(previous.size) * 1.8
        && !large_gap_continues_sentence(previous, next)
}

fn pages_to_text(pages: &[Vec<StyledLine>]) -> String {
    let body_size = body_text_size(pages);
    let mut text = String::new();
    let mut previous_page_last: Option<&StyledLine> = None;
    for page in pages {
        let lines = page
            .iter()
            .filter(|line| !line.text().trim().is_empty())
            .collect::<Vec<_>>();
        let Some(first) = lines.first() else {
            previous_page_last = None;
            continue;
        };
        if let Some(previous) = previous_page_last
            && !text.is_empty()
        {
            let separator = if page_boundary_continues(previous, first, body_size) {
                "\n"
            } else {
                "\n\n"
            };
            text.push_str(separator);
        }
        if !text.is_empty() && previous_page_last.is_none() {
            text.push_str("\n\n");
        }
        for (index, line) in lines.iter().enumerate() {
            if index > 0 {
                let previous = lines[index - 1];
                let separator = if large_gap_starts_paragraph(previous, line) {
                    "\n\n"
                } else {
                    "\n"
                };
                text.push_str(separator);
            }
            text.push_str(&line.text());
        }
        previous_page_last = lines.last().copied();
    }
    text
}

fn merge_wrapped_title_lines(pages: &mut [Vec<StyledLine>]) {
    let all_lines = pages
        .iter()
        .flatten()
        .filter(|line| !line.text().trim().is_empty())
        .collect::<Vec<_>>();
    if all_lines.is_empty() {
        return;
    }

    let mut sizes = all_lines.iter().map(|line| line.size).collect::<Vec<_>>();
    sizes.sort_by(f32::total_cmp);
    let mut size_counts = Vec::<(f32, usize)>::new();
    for size in sizes {
        if let Some((_, count)) = size_counts
            .iter_mut()
            .find(|(known, _)| (size - *known).abs() < 0.5)
        {
            *count += 1;
        } else {
            size_counts.push((size, 1));
        }
    }
    let body_size = size_counts
        .iter()
        .map(|(size, _)| *size)
        .min_by(f32::total_cmp)
        .unwrap_or(12.0)
        .max(1.0);

    for page in pages {
        let mut line_index = 0;
        while line_index + 1 < page.len() {
            let previous = &page[line_index];
            let next = &page[line_index + 1];
            let is_heading_sized = |line: &StyledLine| {
                line.text().chars().count() <= 160 && line.size >= body_size * 1.08
            };
            let similar_size =
                (previous.size - next.size).abs() <= previous.size.max(next.size) * 0.05;
            let tight_leading = next.gap_before <= previous.size * 1.55;
            let matching_style = last_content_style(previous)
                .zip(first_content_style(next))
                .is_some_and(|(before, after)| before == after);
            if !(is_heading_sized(previous)
                && is_heading_sized(next)
                && similar_size
                && tight_leading
                && matching_style)
            {
                line_index += 1;
                continue;
            }

            let next = page.remove(line_index + 1);
            let previous = &mut page[line_index];
            push_line_separator(&mut previous.runs, &next.runs);
            append_runs(&mut previous.runs, &next.runs);
            previous.size = previous.size.max(next.size);
        }
    }
}

fn strip_sponsr_preamble(page: &mut Vec<StyledLine>) {
    let Some(masthead_index) = page
        .iter()
        .take(6)
        .position(|line| line.text().trim().to_lowercase().replace('ё', "е") == "sponsr")
    else {
        return;
    };
    let body_size = page
        .iter()
        .filter(|line| line.text().chars().count() >= 40)
        .map(|line| line.size)
        .min_by(f32::total_cmp)
        .or_else(|| page.iter().map(|line| line.size).min_by(f32::total_cmp))
        .unwrap_or(12.0);
    let Some(title_index) =
        page.iter()
            .enumerate()
            .skip(masthead_index + 1)
            .find_map(|(index, line)| {
                let text = line.text();
                let is_title = text.chars().count() >= 24
                    && (line.size >= body_size * 1.45 || text.trim_start().starts_with('['));
                is_title.then_some(index)
            })
    else {
        return;
    };
    if title_index > 0 {
        page.drain(..title_index);
    }
}

fn strip_sponsr_branding(pages: &mut [Vec<StyledLine>]) {
    for page in pages {
        page.retain(|line| !is_sponsr_branding_line(&line.text()));
    }
}

fn is_sponsr_branding_line(text: &str) -> bool {
    let normalized = text
        .trim()
        .to_lowercase()
        .replace(['\u{2013}', '\u{2014}', '\u{2212}'], "-")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    normalized == "кримсон аналитика - crimsonanalytics"
}

fn lines_to_html(pages: &[Vec<StyledLine>]) -> String {
    let mut grouped_pages = pages.to_vec();
    merge_wrapped_title_lines(&mut grouped_pages);
    let pages = grouped_pages.as_slice();
    let all_lines = pages
        .iter()
        .flatten()
        .filter(|line| !line.text().trim().is_empty())
        .collect::<Vec<_>>();
    if all_lines.is_empty() {
        return String::new();
    }

    let body_size = body_text_size(pages);
    let heading_candidates = all_lines
        .iter()
        .enumerate()
        .filter(|(_, line)| is_heading_candidate(line, body_size))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let title_index = heading_candidates.first().copied();
    let mut heading_sizes = heading_candidates
        .iter()
        .filter_map(|index| {
            if Some(*index) == title_index {
                None
            } else {
                Some(all_lines[*index].size)
            }
        })
        .collect::<Vec<_>>();
    heading_sizes.sort_by(|a, b| b.total_cmp(a));
    heading_sizes.dedup_by(|a, b| (*a - *b).abs() < 0.5);

    let base_left = all_lines
        .iter()
        .map(|line| line.left)
        .min_by(f32::total_cmp)
        .unwrap_or(0.0);
    let mut blocks: Vec<(String, Vec<StyledRun>, bool)> = Vec::new();
    let mut paragraph = Vec::<StyledRun>::new();
    let mut paragraph_quote = false;
    let mut quote_left = None::<f32>;
    let mut global_index = 0usize;
    let mut previous_page_last: Option<&StyledLine> = None;
    for page in pages {
        let lines = page
            .iter()
            .filter(|line| !line.text().trim().is_empty())
            .collect::<Vec<_>>();
        let Some(first_line) = lines.first() else {
            if !paragraph.is_empty() {
                blocks.push((
                    "p".to_owned(),
                    std::mem::take(&mut paragraph),
                    paragraph_quote,
                ));
            }
            previous_page_last = None;
            continue;
        };
        let page_continues = previous_page_last
            .is_some_and(|previous| page_boundary_continues(previous, first_line, body_size));
        if !paragraph.is_empty() && !page_continues {
            blocks.push((
                "p".to_owned(),
                std::mem::take(&mut paragraph),
                paragraph_quote,
            ));
        }
        let page_last = lines.last().copied();
        let mut previous_line: Option<&StyledLine> = None;
        for line in lines {
            if line.left >= base_left + 5.0 {
                quote_left = Some(line.left);
            } else if quote_left.is_some_and(|left| line.left < left - 2.0) {
                quote_left = None;
            }
            let in_quote = quote_left.is_some();
            let line_index = global_index;
            global_index += 1;
            let heading_tag = if Some(line_index) == title_index {
                Some("h1")
            } else if heading_candidates.contains(&line_index) {
                let level = heading_sizes
                    .iter()
                    .position(|size| (line.size - *size).abs() < 0.5)
                    .unwrap_or(usize::MAX);
                match level {
                    0 => Some("h2"),
                    1 => Some("h3"),
                    _ => None,
                }
            } else {
                None
            };

            if let Some(tag) = heading_tag {
                if !paragraph.is_empty() {
                    blocks.push((
                        "p".to_owned(),
                        std::mem::take(&mut paragraph),
                        paragraph_quote,
                    ));
                }
                blocks.push((tag.to_owned(), line.runs.clone(), in_quote));
            } else {
                let paragraph_break = previous_line
                    .is_some_and(|previous| large_gap_starts_paragraph(previous, line));
                if (paragraph_break || (!paragraph.is_empty() && paragraph_quote != in_quote))
                    && !paragraph.is_empty()
                {
                    blocks.push((
                        "p".to_owned(),
                        std::mem::take(&mut paragraph),
                        paragraph_quote,
                    ));
                }
                if paragraph.is_empty() {
                    paragraph_quote = in_quote;
                }
                push_line_separator(&mut paragraph, &line.runs);
                append_runs(&mut paragraph, &line.runs);
            }
            previous_line = Some(line);
        }
        previous_page_last = page_last;
    }
    if !paragraph.is_empty() {
        blocks.push(("p".to_owned(), paragraph, paragraph_quote));
    }

    let mut output = Vec::new();
    let mut index = 0;
    while index < blocks.len() {
        let (tag, runs, in_quote) = &blocks[index];
        if *in_quote {
            let mut quote = Vec::new();
            while index < blocks.len() && blocks[index].2 {
                let (tag, runs, _) = &blocks[index];
                quote.push(format!("<{tag}>{}</{tag}>", styled_runs_html(runs)));
                index += 1;
            }
            output.push(format!("<blockquote>{}</blockquote>", quote.join("\n")));
        } else {
            output.push(format!("<{tag}>{}</{tag}>", styled_runs_html(runs)));
            index += 1;
        }
    }
    output.join("\n")
}

pub fn classify_promotion(filename: &str, first_page: &str) -> &'static str {
    let name = filename.to_lowercase().replace('ё', "е");
    let page = first_page.to_lowercase().replace('ё', "е");
    let explicit_signals = [
        "если вы еще не в клубе",
        "рекламный материал",
        "реклама",
        "join the club",
        "subscribe now",
    ];
    if explicit_signals
        .iter()
        .any(|signal| name.contains(signal) || page.contains(signal))
    {
        "promotional"
    } else {
        "publish"
    }
}

#[cfg(test)]
mod tests {
    use super::{
        StyledLine, StyledRun, cache_fingerprint, extract_pdf, is_skladchik_promotion,
        is_skladchik_purchase_promotion, is_xml_10_char, large_gap_continues_sentence,
        lines_to_html, normalize_text, pages_to_text, push_styled_char, strip_page_label_lines,
        strip_page_labels_by_content_position, strip_sponsr_branding, strip_sponsr_preamble,
        text_to_safe_html,
    };
    use rand::{Rng, RngCore, SeedableRng, rngs::StdRng};

    const AD: &str = "skladchik.ORG\nТы еще\nне в Клубе?\nЧто такое Клуб «Складчик»?";

    #[test]
    fn promotion_match_ignores_case_quotes_and_line_wraps() {
        assert!(is_skladchik_promotion(AD));
    }

    #[test]
    fn promotion_match_requires_all_distinctive_signals() {
        assert!(!is_skladchik_promotion(
            "skladchik.ORG and Ты еще не в Клубе?"
        ));
    }

    #[test]
    fn purchase_ad_requires_leading_purchase_phrase_and_skladchik_domain() {
        assert!(is_skladchik_purchase_promotion(
            "КУПЛЕНО НА\n\nskladchik.ORG"
        ));
        assert!(!is_skladchik_purchase_promotion(
            "Article text mentions skladchik.ORG but not the purchase banner"
        ));
        assert!(!is_skladchik_purchase_promotion(
            "КУПЛЕНО НА another.example"
        ));
    }

    #[test]
    fn strips_sponsr_chrome_before_identifiable_article_title() {
        let mut page = vec![
            line("sponsr", 24.0),
            line("CrimsonAlter – Analytics", 12.0),
            line("27 февр 11:44", 10.0),
            line("58 минут", 10.0),
            line("[Философское] «Старые деньги» — FAQ:", 22.0),
            line("как правильно считать дивиденды", 22.0),
            line("Body paragraph starts here.", 12.0),
        ];
        strip_sponsr_preamble(&mut page);
        assert_eq!(page[0].text(), "[Философское] «Старые деньги» — FAQ:");
        assert_eq!(page.len(), 3);
    }

    #[test]
    fn keeps_content_when_sponsr_title_is_ambiguous() {
        let mut page = vec![line("sponsr", 24.0), line("Short text", 12.0)];
        strip_sponsr_preamble(&mut page);
        assert_eq!(page.len(), 2);
    }

    #[test]
    fn strips_sponsr_branding_from_all_pages_but_preserves_article_mentions() {
        let mut pages = vec![
            vec![
                line("Article before branding.", 12.0),
                StyledLine {
                    runs: ["Кримсон", " Аналитика — ", "CrimsonAnalytics"]
                        .into_iter()
                        .map(|text| StyledRun {
                            text: text.into(),
                            bold: false,
                            italic: false,
                        })
                        .collect(),
                    size: 9.0,
                    gap_before: 0.0,
                    left: 50.0,
                },
                line("Article after branding.", 12.0),
                line("CrimsonAlter – Analytics", 9.0),
            ],
            vec![
                line(
                    "The article discusses Кримсон Аналитика – CrimsonAnalytics in context.",
                    12.0,
                ),
                line("Кримсон Аналитика – CrimsonAnalytics", 9.0),
            ],
        ];

        strip_sponsr_branding(&mut pages);

        let text = pages
            .iter()
            .flatten()
            .map(StyledLine::text)
            .collect::<Vec<_>>()
            .join("\n");
        let html = lines_to_html(&pages);
        assert!(text.contains("Article before branding."));
        assert!(text.contains("Article after branding."));
        assert!(
            text.contains("The article discusses Кримсон Аналитика – CrimsonAnalytics in context.")
        );
        assert!(text.contains("CrimsonAlter – Analytics"));
        assert_eq!(text.matches("Кримсон Аналитика").count(), 1);
        assert_eq!(html.matches("Кримсон Аналитика").count(), 1);
        assert!(!html.contains("Кримсон Аналитика — CrimsonAnalytics</p>"));
    }

    #[test]
    fn strips_branding_from_the_supplied_pdf_opening_sequence() {
        let mut pages = vec![vec![
            line(
                "Предыдущий апдейт тут -> sponsr.ru/crimsonanalytics/172225",
                10.0,
            ),
            line("Кримсон Аналитика – CrimsonAnalytics", 9.0),
            line("Традиционно напоминаю несколько моментов:", 12.0),
        ]];

        strip_sponsr_branding(&mut pages);

        let html = lines_to_html(&pages);
        assert_eq!(
            html,
            "<p>Предыдущий апдейт тут -&gt; sponsr.ru/crimsonanalytics/172225</p>\n<h1>Традиционно напоминаю несколько моментов:</h1>"
        );
        assert!(!html.contains("Кримсон Аналитика"));
    }

    fn line(text: &str, size: f32) -> StyledLine {
        StyledLine {
            runs: vec![StyledRun {
                text: text.into(),
                bold: false,
                italic: false,
            }],
            size,
            gap_before: 0.0,
            left: 50.0,
        }
    }

    #[test]
    fn large_gap_continuation_requires_continuation_punctuation() {
        let previous = line(
            "Нужно всей планетой платить очень много денег зелёным спасителям,",
            15.0,
        );
        let next = line(
            "а доказательством их эффективности будет то, что напророченная",
            15.0,
        );
        assert!(large_gap_continues_sentence(&previous, &next));

        let completed = line("Первый абзац закончен.", 15.0);
        assert!(!large_gap_continues_sentence(&completed, &next));
    }

    #[test]
    fn large_gap_continuation_recognizes_honorific_before_uppercase_text() {
        for (previous, next) in [
            (
                "Понедельник, ув. друзья, и в кабине «Орбитального экспресса» ув.",
                "Александр Романов с захватывающим вопросом о вопросах.",
            ),
            (
                "В первую очередь стоит отметить ценность стартового тезиса ув.",
                "Собеседника. Идея «геноцида неизвестности» соответствует.",
            ),
        ] {
            assert!(large_gap_continues_sentence(
                &line(previous, 15.0),
                &line(next, 15.0)
            ));
        }

        assert!(!large_gap_continues_sentence(
            &line("Первый абзац полностью завершен.", 15.0),
            &line("Следующий абзац начинается здесь.", 15.0),
        ));
        assert!(!large_gap_continues_sentence(
            &line("Составное слово содержитнеув.", 15.0),
            &line("Продолжение начинается здесь.", 15.0),
        ));
    }

    #[test]
    fn large_gap_continuation_rejects_incompatible_alignment_and_style() {
        let previous = line("A sentence continues,", 15.0);
        let mut next = line("with matching layout.", 15.0);

        next.left = 60.0;
        assert!(!large_gap_continues_sentence(&previous, &next));

        next.left = previous.left;
        next.runs[0].bold = true;
        assert!(!large_gap_continues_sentence(&previous, &next));
    }

    #[test]
    fn strips_matching_page_number_wherever_it_appears_in_extracted_order() {
        let mut leading = vec![line("  2  ", 10.0), line("Article starts here", 12.0)];
        strip_page_label_lines(&mut leading, 2);
        assert_eq!(
            leading.iter().map(StyledLine::text).collect::<Vec<_>>(),
            ["Article starts here"]
        );

        let mut later = vec![line("Article", 12.0), line("2", 10.0), line("More", 12.0)];
        strip_page_label_lines(&mut later, 2);
        assert_eq!(
            later.iter().map(StyledLine::text).collect::<Vec<_>>(),
            ["Article", "More"]
        );
    }

    #[test]
    fn preserves_nonmatching_or_body_numbers() {
        let mut lines = vec![line("3", 10.0), line("Page 2", 12.0), line("2 items", 12.0)];
        strip_page_label_lines(&mut lines, 2);
        assert_eq!(
            lines.iter().map(StyledLine::text).collect::<Vec<_>>(),
            ["3", "Page 2", "2 items"]
        );
    }

    #[test]
    fn strips_labels_by_content_position_when_leading_ad_shifts_printed_numbering() {
        let mut pages = vec![
            vec![line("1", 11.0), line("Title", 22.0)],
            vec![
                line("First content page body", 12.0),
                line("2 items", 12.0),
                line("2", 11.0),
            ],
            vec![line("3", 11.0), line("Second content page body", 12.0)],
        ];

        strip_page_labels_by_content_position(&mut pages);

        assert_eq!(
            pages[0].iter().map(StyledLine::text).collect::<Vec<_>>(),
            ["Title"]
        );
        assert_eq!(
            pages[1].iter().map(StyledLine::text).collect::<Vec<_>>(),
            ["First content page body", "2 items"]
        );
        assert_eq!(
            pages[2].iter().map(StyledLine::text).collect::<Vec<_>>(),
            ["Second content page body"]
        );
    }

    #[test]
    fn joins_wrapped_lines_inside_a_paragraph() {
        let input = "Четверг, ув. друзья, и самое время вступить на офигенно скользкий лёд и\nпройтись под его весёлый хруст по настоящему и будущему великой\nстраны.";
        assert_eq!(
            normalize_text(input),
            "Четверг, ув. друзья, и самое время вступить на офигенно скользкий лёд и пройтись под его весёлый хруст по настоящему и будущему великой страны."
        );
    }

    #[test]
    fn preserves_paragraph_heading_and_list_boundaries() {
        let input =
            "Heading\n\nFirst paragraph wraps\nover two lines.\n\n- First item\n\n- Second item";
        assert_eq!(
            normalize_text(input),
            "Heading\n\nFirst paragraph wraps over two lines.\n\n- First item\n\n- Second item"
        );
    }

    #[test]
    fn safe_html_uses_reflowed_paragraphs_and_keeps_escaping() {
        let normalized =
            normalize_text("First <span> wraps\nline & continues.\n\nSecond paragraph.");
        assert_eq!(
            text_to_safe_html(&normalized),
            "<p>First &lt;span&gt; wraps line &amp; continues.</p>\n<p>Second paragraph.</p>"
        );
    }

    #[test]
    fn rejects_xml_1_0_forbidden_control_characters() {
        assert!(!is_xml_10_char('\u{2}'));
        assert!(is_xml_10_char('\t'));
        assert!(is_xml_10_char('ё'));
        assert!(is_xml_10_char('𐀀'));
    }

    #[test]
    fn generated_text_keeps_normalization_and_safe_html_invariants() {
        let mut rng = StdRng::seed_from_u64(0x004b_5041_5045_5253);
        for _ in 0..512 {
            let mut bytes = vec![0; rng.gen_range(0..512)];
            rng.fill_bytes(&mut bytes);
            let raw = String::from_utf8_lossy(&bytes);
            let normalized = normalize_text(&raw);

            assert_eq!(normalize_text(&normalized), normalized);
            assert!(!normalized.contains('\0'));
            assert!(!normalized.contains('\u{fffd}'));

            let html = text_to_safe_html(&normalized);
            for fragment in html.split('<').skip(1) {
                let (tag, _) = fragment.split_once('>').expect("well-formed generated tag");
                assert!(matches!(tag, "p" | "/p"), "unexpected tag <{tag}>");
            }
            assert!(!html.contains("<script"));
            assert!(!html.contains("<img"));
            assert!(matches!(
                super::classify_promotion(&raw, &normalized),
                "publish" | "promotional"
            ));
        }
    }

    #[test]
    fn pdf_entry_point_rejects_non_pdf_and_oversized_data_before_parsing() {
        assert!(matches!(
            extract_pdf(b"not a pdf", 1024),
            Err(error) if error == "source is not a PDF file"
        ));
        assert!(matches!(
            extract_pdf(b"%PDF-too-large", 4),
            Err(error) if error == "PDF exceeds configured size limit"
        ));
    }

    fn simple_pdf_pages(texts: &[&str]) -> Vec<u8> {
        let font_id = 3 + 2 * texts.len();
        let mut objects = vec![
            "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
            format!(
                "<< /Type /Pages /Kids [{}] /Count {} >>",
                (0..texts.len())
                    .map(|index| format!("{} 0 R", 3 + 2 * index))
                    .collect::<Vec<_>>()
                    .join(" "),
                texts.len()
            ),
        ];
        for (index, text) in texts.iter().enumerate() {
            let page_id = 3 + 2 * index;
            let content_id = page_id + 1;
            objects.push(format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 {font_id} 0 R >> >> /Contents {content_id} 0 R >>"
            ));
            let content = format!("BT /F1 12 Tf 50 700 Td ({text}) Tj ET");
            objects.push(format!(
                "<< /Length {} >>\nstream\n{content}\nendstream",
                content.len() + 1
            ));
        }
        objects.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_owned());
        let mut output = b"%PDF-1.4\n".to_vec();
        let mut offsets = Vec::new();
        for (index, object) in objects.iter().enumerate() {
            offsets.push(output.len());
            output.extend(format!("{} 0 obj\n{object}\nendobj\n", index + 1).as_bytes());
        }
        let xref = output.len();
        output.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes());
        for offset in offsets {
            output.extend(format!("{offset:010} 00000 n \n").as_bytes());
        }
        output.extend(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
                objects.len() + 1
            )
            .as_bytes(),
        );
        output
    }

    #[test]
    fn pdf_oxide_default_text_extraction_preserves_page_text_and_safe_html() {
        let extracted = super::extract_pdf(
            &simple_pdf_pages(&["First page text", "Second page text"]),
            1024 * 1024,
        )
        .unwrap();
        assert_eq!(extracted.page_count, 2);
        assert!(
            extracted.full_text.contains("First page text"),
            "{}",
            extracted.full_text
        );
        assert!(
            extracted.full_text.contains("Second page text"),
            "{}",
            extracted.full_text
        );
        assert!(
            extracted.full_text.find("First page text").unwrap()
                < extracted.full_text.find("Second page text").unwrap()
        );
        assert!(
            extracted.html.contains("First page text"),
            "{}",
            extracted.html
        );
        assert!(
            extracted.html.contains("Second page text"),
            "{}",
            extracted.html
        );
        assert!(!extracted.html.contains("<script"));
    }

    #[test]
    fn pdf_without_an_extractable_text_layer_fails_without_ocr() {
        let error = super::extract_pdf(&simple_pdf_pages(&[""]), 1024 * 1024)
            .expect_err("an empty text layer must not publish an empty article");
        assert!(error.contains("no usable native text"), "{error}");
    }

    #[test]
    fn restores_sponsr_heading_and_removes_publication_metadata_from_document_name() {
        let mut extracted = super::ExtractedText {
            full_text: "27 февр 11:44 58 минут Сразу к делу, коллеги.".into(),
            html: "<h1>27 февр 11:44 58 минут</h1>\n<h2>Сразу к делу, коллеги.</h2>".into(),
            first_page: "27 февр 11:44 58 минут Сразу к делу, коллеги.".into(),
            page_count: 1,
        };
        super::repair_sponsr_title_from_filename(
            "[Философское] «Старые деньги» — FAQ_ как правильно считать дивиденды [sklad.pdf",
            &mut extracted,
        );
        assert_eq!(
            extracted.html,
            "<h1><strong>[Философское] «Старые деньги» — FAQ: как правильно считать дивиденды</strong></h1>\n<p>Сразу к делу, коллеги.</p>"
        );
        assert_eq!(
            extracted.first_page,
            "[Философское] «Старые деньги» — FAQ: как правильно считать дивиденды\nСразу к делу, коллеги."
        );
        assert_eq!(
            extracted.full_text,
            "[Философское] «Старые деньги» — FAQ: как правильно считать дивиденды\n\nСразу к делу, коллеги."
        );
    }

    #[test]
    fn filename_recovered_sponsr_title_keeps_body_blocks_and_clear_section_heading() {
        let mut extracted = super::ExtractedText {
            full_text: "27 февр 11:44 58 минут body".into(),
            html: "<p>27 февр 11:44</p>\n<h1><strong>это не инвестиционные рекомендации</strong></h1>\n<h2>Сразу к делу, коллеги.</h2>\n<h2><strong>Раздел статьи</strong></h2>\n<blockquote><p>Традиционные технические предуведомления:</p></blockquote>\n<p>Обычный текст после цитаты.</p>".into(),
            first_page: "27 февр 11:44 58 минут body".into(),
            page_count: 1,
        };
        super::repair_sponsr_title_from_filename(
            "[Философское] «Старые деньги» — FAQ_ как правильно считать дивиденды.pdf",
            &mut extracted,
        );
        assert!(extracted.html.starts_with("<h1><strong>[Философское] «Старые деньги» — FAQ: как правильно считать дивиденды</strong></h1>"));
        assert!(
            extracted
                .html
                .contains("<p><strong>это не инвестиционные рекомендации</strong></p>")
        );
        assert!(extracted.html.contains("<p>Сразу к делу, коллеги.</p>"));
        assert!(
            extracted
                .html
                .contains("<h2><strong>Раздел статьи</strong></h2>")
        );
        assert!(
            extracted.html.contains(
                "<blockquote><p>Традиционные технические предуведомления:</p></blockquote>"
            )
        );
        assert!(
            extracted
                .html
                .ends_with("<p>Обычный текст после цитаты.</p>")
        );
        assert!(!extracted.html.contains("27 февр 11:44"));
    }

    #[test]
    fn cache_fingerprint_contains_pdf_oxide_and_processing_revision() {
        let source = "source-md5";
        let processed = cache_fingerprint(source);
        assert_eq!(processed, format!("pdf-text-v15:pdf_oxide:{source}"));
        assert_ne!(processed, format!("pdf-text-v14:pdf_oxide:{source}"));
        assert_ne!(processed, format!("pdf-text-v13:pdfium:{source}"));
        assert_ne!(processed, format!("pdf-text-v13:marker:{source}"));
        assert_ne!(processed, source);
        assert_eq!(cache_fingerprint(source), processed);
    }

    #[test]
    fn renders_bold_italic_combined_and_unstyled_runs_safely() {
        let mut runs = Vec::new();
        push_styled_char(&mut runs, '<', true, false);
        let runs = vec![
            runs.pop()
                .map(|mut run| {
                    run.text.push_str("bold>");
                    run
                })
                .unwrap(),
            StyledRun {
                text: "italic".into(),
                bold: false,
                italic: true,
            },
            StyledRun {
                text: "both".into(),
                bold: true,
                italic: true,
            },
            StyledRun {
                text: "plain & safe".into(),
                bold: false,
                italic: false,
            },
        ];
        assert_eq!(
            super::styled_runs_html(&runs),
            "<strong>&lt;bold&gt;</strong><em>italic<strong>both</strong></em>plain &amp; safe"
        );
    }

    #[test]
    fn wraps_indented_technical_notice_blocks_in_one_blockquote() {
        let mut notice = line("Традиционные технические предуведомления:", 12.0);
        notice.left = 70.0;
        let mut continuation = line("Для новых читателей: начинайте с этого текста.", 12.0);
        continuation.left = 70.0;
        continuation.gap_before = 32.0;
        let mut following = line("Обычный текст после предуведомления.", 12.0);
        following.left = 50.0;
        following.gap_before = 32.0;
        let html = lines_to_html(&[vec![
            line("Начало статьи.", 12.0),
            notice,
            continuation,
            following,
        ]]);
        assert!(html.contains(
            "<blockquote><p>Традиционные технические предуведомления:</p>\n<p>Для новых читателей: начинайте с этого текста.</p></blockquote>"
        ), "{html}");
        assert!(
            html.ends_with("<p>Обычный текст после предуведомления.</p>"),
            "{html}"
        );
    }

    #[test]
    fn joins_a_prose_paragraph_continuing_across_pages() {
        let pages = vec![
            vec![line(
                "Вот такой-то псих с накаченной на него поверх системой ложных представлений о себе — это третий вариант",
                12.0,
            )],
            vec![line(
                "возможного гражданина вашей утопии. Или антиутопии, или хрен вас знает, сами разбирайтесь.",
                12.0,
            )],
        ];
        let expected = "Вот такой-то псих с накаченной на него поверх системой ложных представлений о себе — это третий вариант возможного гражданина вашей утопии. Или антиутопии, или хрен вас знает, сами разбирайтесь.";

        assert_eq!(normalize_text(&pages_to_text(&pages)), expected);
        assert_eq!(lines_to_html(&pages), format!("<p>{expected}</p>"));
    }

    #[test]
    fn preserves_a_real_paragraph_boundary_across_pages() {
        let pages = vec![
            vec![line("Первый абзац полностью завершен.", 12.0)],
            vec![line("Следующий абзац начинается здесь.", 12.0)],
        ];

        assert_eq!(
            normalize_text(&pages_to_text(&pages)),
            "Первый абзац полностью завершен.\n\nСледующий абзац начинается здесь."
        );
        assert_eq!(
            lines_to_html(&pages),
            "<p>Первый абзац полностью завершен.</p>\n<p>Следующий абзац начинается здесь.</p>"
        );
    }

    #[test]
    fn joins_sentence_continuation_across_large_within_page_gap_in_text_and_html() {
        let first = line(
            "Нужно всей планетой платить очень много денег зелёным спасителям,",
            15.0,
        );
        let mut continuation = line(
            "а доказательством их эффективности будет то, что напророченная ими же всеобщая гибель не состоится.",
            15.0,
        );
        continuation.gap_before = 39.0;
        let pages = vec![vec![first, continuation]];
        let expected = "Нужно всей планетой платить очень много денег зелёным спасителям, а доказательством их эффективности будет то, что напророченная ими же всеобщая гибель не состоится.";

        assert_eq!(normalize_text(&pages_to_text(&pages)), expected);
        assert_eq!(lines_to_html(&pages), format!("<p>{expected}</p>"));
    }

    #[test]
    fn joins_reported_honorific_continuations_in_text_and_html() {
        let cases = [
            (
                "Понедельник, ув. друзья, и в кабине «Орбитального экспресса» ув.",
                "Александр Романов с захватывающим вопросом о вопросах.",
            ),
            (
                "В первую очередь стоит отметить ценность стартового тезиса ув.",
                "Собеседника. Идея «геноцида неизвестности», наступившего сейчас, безусловно соответствует",
            ),
        ];

        for (first, second) in cases {
            let mut continuation = line(second, 15.0);
            continuation.gap_before = 39.0;
            let pages = vec![vec![line(first, 15.0), continuation]];
            let expected = format!("{first} {second}");

            assert_eq!(normalize_text(&pages_to_text(&pages)), expected);
            assert_eq!(lines_to_html(&pages), format!("<p>{expected}</p>"));
        }
    }

    #[test]
    fn keeps_large_gap_after_completed_sentence_as_paragraph_boundary() {
        let first = line("Первый абзац полностью завершен.", 15.0);
        let mut next_paragraph = line("Следующий абзац начинается здесь.", 15.0);
        next_paragraph.gap_before = 39.0;
        let pages = vec![vec![first, next_paragraph]];

        assert_eq!(
            normalize_text(&pages_to_text(&pages)),
            "Первый абзац полностью завершен.\n\nСледующий абзац начинается здесь."
        );
        assert_eq!(
            lines_to_html(&pages),
            "<p>Первый абзац полностью завершен.</p>\n<p>Следующий абзац начинается здесь.</p>"
        );
    }

    #[test]
    fn merges_same_style_runs_across_whitespace_only_run() {
        let runs = vec![
            StyledRun {
                text: "06.11.2026".into(),
                bold: true,
                italic: false,
            },
            StyledRun {
                text: " ".into(),
                bold: false,
                italic: false,
            },
            StyledRun {
                text: "12:12".into(),
                bold: true,
                italic: false,
            },
        ];
        assert_eq!(
            super::styled_runs_html(&runs),
            "<strong>06.11.2026 12:12</strong>"
        );
    }

    #[test]
    fn merges_italic_runs_around_nested_bold_and_keeps_paragraph_breaks() {
        let page = vec![
            StyledLine {
                runs: vec![StyledRun {
                    text: "Attribution text".into(),
                    bold: false,
                    italic: false,
                }],
                size: 12.0,
                gap_before: 0.0,
                left: 50.0,
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "«Техас — крупнейший ".into(),
                    bold: false,
                    italic: true,
                }],
                size: 12.0,
                gap_before: 32.0,
                left: 50.0,
            },
            StyledLine {
                runs: vec![
                    StyledRun {
                        text: "красный".into(),
                        bold: true,
                        italic: true,
                    },
                    StyledRun {
                        text: " штат в стране.".into(),
                        bold: false,
                        italic: true,
                    },
                ],
                size: 12.0,
                gap_before: 16.0,
                left: 50.0,
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "Демократ Джеймс Таларико".into(),
                    bold: false,
                    italic: true,
                }],
                size: 12.0,
                gap_before: 32.0,
                left: 50.0,
            },
        ];
        assert_eq!(
            lines_to_html(&[page]),
            "<p>Attribution text</p>\n<p><em>«Техас — крупнейший <strong>красный</strong> штат в стране.</em></p>\n<p><em>Демократ Джеймс Таларико</em></p>"
        );
    }

    #[test]
    fn wrapped_document_title_is_one_h1() {
        let page = vec![
            StyledLine {
                runs: vec![StyledRun {
                    text: "Ядерный Ливан. Путеводитель по".into(),
                    bold: true,
                    italic: false,
                }],
                size: 22.0,
                gap_before: 0.0,
                left: 50.0,
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "выборам и будущему США".into(),
                    bold: true,
                    italic: false,
                }],
                size: 22.0,
                gap_before: 27.0,
                left: 50.0,
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "Сегодня для нас нет ничего актуальней сдвигов.".into(),
                    bold: false,
                    italic: false,
                }],
                size: 12.0,
                gap_before: 40.0,
                left: 50.0,
            },
        ];
        let html = lines_to_html(&[page]);
        assert!(
            html.starts_with(
                "<h1><strong>Ядерный Ливан. Путеводитель по выборам и будущему США</strong></h1>"
            ),
            "{html}"
        );
        assert!(!html.contains("<h2><strong>выборам"), "{html}");
        assert!(
            html.contains("<p>Сегодня для нас нет ничего актуальней сдвигов.</p>"),
            "{html}"
        );
    }

    #[test]
    fn wrapped_top_level_headline_is_one_h2() {
        let page = vec![
            StyledLine {
                runs: vec![StyledRun {
                    text: "06.11.2026 12:12".into(),
                    bold: true,
                    italic: false,
                }],
                size: 14.0,
                gap_before: 0.0,
                left: 50.0,
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "Дмитриев у Трампов, трудности у".into(),
                    bold: true,
                    italic: false,
                }],
                size: 22.0,
                gap_before: 35.0,
                left: 50.0,
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "танкеров, мегаполисы США у".into(),
                    bold: true,
                    italic: false,
                }],
                size: 22.0,
                gap_before: 26.0,
                left: 50.0,
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "социалистов".into(),
                    bold: true,
                    italic: false,
                }],
                size: 22.0,
                gap_before: 26.0,
                left: 50.0,
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "У Европы кризис жанра".into(),
                    bold: true,
                    italic: false,
                }],
                size: 16.0,
                gap_before: 40.0,
                left: 50.0,
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "Body text".into(),
                    bold: false,
                    italic: false,
                }],
                size: 12.0,
                gap_before: 36.0,
                left: 50.0,
            },
        ];
        let html = lines_to_html(&[page]);
        assert!(
            html.contains("<h1><strong>06.11.2026 12:12</strong></h1>"),
            "{html}"
        );
        assert!(html.contains("<h2><strong>Дмитриев у Трампов, трудности у танкеров, мегаполисы США у социалистов</strong></h2>"), "{html}");
        assert!(
            html.contains("<h3><strong>У Европы кризис жанра</strong></h3>"),
            "{html}"
        );
    }

    #[test]
    fn detects_heading_hierarchy_and_keeps_body_as_paragraphs() {
        let page = vec![
            StyledLine {
                runs: vec![StyledRun {
                    text: "Document title".into(),
                    bold: true,
                    italic: false,
                }],
                size: 22.0,
                gap_before: 0.0,
                left: 50.0,
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "Section".into(),
                    bold: true,
                    italic: false,
                }],
                size: 16.0,
                gap_before: 16.0,
                left: 50.0,
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "Body text".into(),
                    bold: false,
                    italic: false,
                }],
                size: 12.0,
                gap_before: 16.0,
                left: 50.0,
            },
        ];
        let html = lines_to_html(&[page]);
        assert_eq!(
            html,
            "<h1><strong>Document title</strong></h1>\n<h2><strong>Section</strong></h2>\n<p>Body text</p>"
        );
    }

    #[test]
    fn prominent_prose_stays_a_paragraph_without_heading_typography() {
        let page = vec![
            StyledLine {
                runs: vec![StyledRun {
                    text: "Document title".into(),
                    bold: true,
                    italic: false,
                }],
                size: 24.0,
                gap_before: 0.0,
                left: 50.0,
            },
            line("Ordinary body text establishes the common font size.", 12.0),
            StyledLine {
                runs: vec![StyledRun {
                    text: "Здесь уместно вспомнить нашу собственную недавнюю историю, в ходе которой под занавес 90-х как бы из ниоткуда явился В. В. Путин, воспитанник специфической профессиональной школы.".into(),
                    bold: false,
                    italic: false,
                }],
                size: 14.0,
                gap_before: 36.0,
                left: 50.0,
            },
            line("More ordinary body text follows the prominent paragraph.", 12.0),
        ];

        let html = lines_to_html(&[page]);
        assert!(
            html.starts_with("<h1><strong>Document title</strong></h1>"),
            "{html}"
        );
        assert!(
            html.contains("<p>Здесь уместно вспомнить нашу собственную недавнюю историю"),
            "{html}"
        );
        assert!(!html.contains("<h2>Здесь уместно вспомнить"), "{html}");
    }

    #[test]
    fn detects_nested_heading_levels() {
        let page = vec![
            StyledLine {
                runs: vec![StyledRun {
                    text: "Title".into(),
                    bold: true,
                    italic: false,
                }],
                size: 22.0,
                gap_before: 0.0,
                left: 50.0,
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "Section".into(),
                    bold: true,
                    italic: false,
                }],
                size: 16.0,
                gap_before: 16.0,
                left: 50.0,
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "Subsection".into(),
                    bold: true,
                    italic: false,
                }],
                size: 14.0,
                gap_before: 16.0,
                left: 50.0,
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "Nested subsection".into(),
                    bold: false,
                    italic: true,
                }],
                size: 13.0,
                gap_before: 32.0,
                left: 50.0,
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "Body".into(),
                    bold: false,
                    italic: false,
                }],
                size: 12.0,
                gap_before: 32.0,
                left: 50.0,
            },
        ];
        let html = lines_to_html(&[page]);
        assert!(html.contains("<h1><strong>Title</strong></h1>"), "{html}");
        assert!(html.contains("<h2><strong>Section</strong></h2>"), "{html}");
        assert!(
            html.contains("<h3><strong>Subsection</strong></h3>"),
            "{html}"
        );
        assert!(html.contains("<p><em>Nested subsection</em></p>"), "{html}");
        assert!(!html.contains("<h4>"), "{html}");
        assert!(html.ends_with("<p>Body</p>"), "{html}");
    }
}
