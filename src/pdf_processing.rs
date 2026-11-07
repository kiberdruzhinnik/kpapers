use pdfium_render::prelude::{PdfFontWeight, Pdfium};
use std::sync::OnceLock;

pub struct ExtractedText {
    pub full_text: String,
    pub html: String,
    pub first_page: String,
    pub page_count: usize,
}

const EXTRACTION_PIPELINE_REVISION: u32 = 3;

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
}

impl StyledLine {
    fn text(&self) -> String {
        self.runs.iter().map(|run| run.text.as_str()).collect()
    }
}

pub fn cache_fingerprint(source_fingerprint: &str) -> String {
    format!("pdf-text-v{EXTRACTION_PIPELINE_REVISION}:{source_fingerprint}")
}

pub fn extract_pdf(bytes: &[u8], maximum_bytes: usize) -> Result<ExtractedText, String> {
    if bytes.len() > maximum_bytes {
        return Err("PDF exceeds configured size limit".into());
    }
    if !bytes.starts_with(b"%PDF-") {
        return Err("source is not a PDF file".into());
    }

    let pdfium = pdfium()?;
    let document = pdfium
        .load_pdf_from_byte_slice(bytes, None)
        .map_err(|error| format!("PDF document load failed: {error}"))?;
    let page_count = document.pages().len() as usize;
    if page_count == 0 {
        return Err("PDF contains no pages".into());
    }

    let mut pages = Vec::with_capacity(page_count);
    for page_index in 0..page_count {
        let page = document
            .pages()
            .get(page_index as _)
            .map_err(|error| format!("PDF page load failed: {error}"))?;
        let text = page
            .text()
            .map_err(|error| format!("PDF text extraction failed: {error}"))?;
        let chars = text.chars();
        let mut lines: Vec<StyledLine> = Vec::new();
        let mut previous_y: Option<f32> = None;
        let mut previous_size = 0.0f32;
        let mut force_new_line = false;

        for character in chars.iter() {
            let Some(ch) = character.unicode_char() else {
                continue;
            };
            if ch == '\r' {
                continue;
            }
            let y = character
                .origin_y()
                .map_err(|error| format!("PDF character position extraction failed: {error}"))?
                .value;
            let size = character.scaled_font_size().value.max(1.0);
            if ch == '\n' {
                force_new_line = true;
                continue;
            }

            let line_break = force_new_line
                || previous_y
                    .is_some_and(|last_y| (last_y - y).abs() > size.max(previous_size) * 0.5);
            if line_break || lines.is_empty() {
                let gap_before = previous_y.map(|last_y| (last_y - y).abs()).unwrap_or(0.0);
                lines.push(StyledLine {
                    runs: Vec::new(),
                    size,
                    gap_before,
                });
            }
            force_new_line = false;

            let weight = character.font_weight();
            let font_name = character.font_name().to_lowercase();
            let bold = matches!(
                weight,
                Some(
                    PdfFontWeight::Weight600
                        | PdfFontWeight::Weight700Bold
                        | PdfFontWeight::Weight800
                        | PdfFontWeight::Weight900
                )
            ) || font_name.contains("bold")
                || font_name.contains("black")
                || font_name.contains("demi");
            let italic = character.font_is_italic()
                || font_name.contains("italic")
                || font_name.contains("oblique");
            let line = lines.last_mut().expect("a line was created");
            line.size = line.size.max(size);
            push_styled_char(&mut line.runs, ch, bold, italic);
            previous_y = Some(y);
            previous_size = size;
        }

        for line in &mut lines {
            trim_run_edges(&mut line.runs);
        }
        lines.retain(|line| !line.text().trim().is_empty());
        if let Some(first) = lines.iter().position(|line| !line.text().trim().is_empty()) {
            if lines[first].text().trim() == (page_index + 1).to_string() {
                lines.remove(first);
            }
        }
        pages.push(lines);
    }

    if pages.last().is_some_and(|lines| {
        is_skladchik_promotion(
            &lines
                .iter()
                .map(StyledLine::text)
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }) {
        pages.pop();
    }

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
    let page_text = pages
        .iter()
        .map(|lines| {
            lines
                .iter()
                .map(StyledLine::text)
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n\n");
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

fn pdfium() -> Result<&'static Pdfium, String> {
    static INSTANCE: OnceLock<Result<Pdfium, String>> = OnceLock::new();
    match INSTANCE.get_or_init(|| {
        let path = std::env::var_os("PDFIUM_LIBRARY_PATH")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| Pdfium::pdfium_platform_library_name_at_path("/usr/local/lib"));
        let bindings = Pdfium::bind_to_library(path)
            .map_err(|error| format!("Unable to load Pdfium runtime: {error}"))?;
        Ok(Pdfium::new(bindings))
    }) {
        Ok(pdfium) => Ok(pdfium),
        Err(error) => Err(error.clone()),
    }
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

#[cfg(test)]
fn strip_page_label(page: &str, page_number: usize) -> String {
    let expected = page_number.to_string();
    let mut first_nonempty_seen = false;
    page.lines()
        .filter_map(|line| {
            if !first_nonempty_seen && !line.trim().is_empty() {
                first_nonempty_seen = true;
                if line.trim() == expected {
                    return None;
                }
            }
            Some(line)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_skladchik_promotion(text: &str) -> bool {
    let normalized = text
        .to_lowercase()
        .replace('ё', "е")
        .chars()
        .map(|ch| if ch.is_alphanumeric() { ch } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    normalized.contains("skladchik org")
        && normalized.contains("ты еще не в клубе")
        && normalized.contains("что такое клуб складчик")
}

pub fn normalize_text(input: &str) -> String {
    let clean = input
        .replace('\0', "")
        .replace('\u{fffd}', "")
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

fn trim_run_edges(runs: &mut Vec<StyledRun>) {
    if let Some(first) = runs.first_mut() {
        first.text = first.text.trim_start().to_owned();
    }
    if let Some(last) = runs.last_mut() {
        last.text = last.text.trim_end().to_owned();
    }
    runs.retain(|run| !run.text.is_empty());
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
    let heading_candidates = all_lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.text().chars().count() <= 160 && line.size >= body_size * 1.08)
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

    let mut blocks: Vec<(String, Vec<StyledRun>)> = Vec::new();
    let mut paragraph = Vec::<StyledRun>::new();
    let mut global_index = 0usize;
    for page in pages {
        if !paragraph.is_empty() {
            blocks.push(("p".to_owned(), std::mem::take(&mut paragraph)));
        }
        let mut previous_line: Option<&StyledLine> = None;
        for line in page.iter().filter(|line| !line.text().trim().is_empty()) {
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
                    blocks.push(("p".to_owned(), std::mem::take(&mut paragraph)));
                }
                blocks.push((tag.to_owned(), line.runs.clone()));
            } else {
                let paragraph_break = previous_line
                    .is_some_and(|previous| line.gap_before > line.size.max(previous.size) * 1.8);
                if paragraph_break && !paragraph.is_empty() {
                    blocks.push(("p".to_owned(), std::mem::take(&mut paragraph)));
                }
                push_line_separator(&mut paragraph, &line.runs);
                append_runs(&mut paragraph, &line.runs);
            }
            previous_line = Some(line);
        }
    }
    if !paragraph.is_empty() {
        blocks.push(("p".to_owned(), paragraph));
    }

    blocks
        .iter()
        .map(|(tag, runs)| format!("<{tag}>{}</{tag}>", styled_runs_html(runs)))
        .collect::<Vec<_>>()
        .join("\n")
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
        StyledLine, StyledRun, cache_fingerprint, is_skladchik_promotion, lines_to_html,
        normalize_text, push_styled_char, strip_page_label, text_to_safe_html,
    };

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
    fn strips_matching_number_from_first_nonempty_line() {
        assert_eq!(
            strip_page_label("\n  2  \nArticle starts here", 2),
            "\nArticle starts here"
        );
    }

    #[test]
    fn preserves_nonmatching_or_body_numbers() {
        assert_eq!(strip_page_label("3\nArticle", 2), "3\nArticle");
        assert_eq!(strip_page_label("Article\n2\nMore", 2), "Article\n2\nMore");
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
    fn cache_fingerprint_invalidates_old_entries_and_is_stable() {
        let source = "source-md5";
        let processed = cache_fingerprint(source);
        assert!(processed.starts_with("pdf-text-v3:"));
        assert_ne!(processed, format!("pdf-text-v2:{source}"));
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
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "«Техас — крупнейший ".into(),
                    bold: false,
                    italic: true,
                }],
                size: 12.0,
                gap_before: 32.0,
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
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "Демократ Джеймс Таларико".into(),
                    bold: false,
                    italic: true,
                }],
                size: 12.0,
                gap_before: 32.0,
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
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "выборам и будущему США".into(),
                    bold: true,
                    italic: false,
                }],
                size: 22.0,
                gap_before: 27.0,
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "Сегодня для нас нет ничего актуальней сдвигов.".into(),
                    bold: false,
                    italic: false,
                }],
                size: 12.0,
                gap_before: 40.0,
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
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "Дмитриев у Трампов, трудности у".into(),
                    bold: true,
                    italic: false,
                }],
                size: 22.0,
                gap_before: 35.0,
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "танкеров, мегаполисы США у".into(),
                    bold: true,
                    italic: false,
                }],
                size: 22.0,
                gap_before: 26.0,
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "социалистов".into(),
                    bold: true,
                    italic: false,
                }],
                size: 22.0,
                gap_before: 26.0,
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "У Европы кризис жанра".into(),
                    bold: true,
                    italic: false,
                }],
                size: 16.0,
                gap_before: 40.0,
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "Body text".into(),
                    bold: false,
                    italic: false,
                }],
                size: 12.0,
                gap_before: 36.0,
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
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "Section".into(),
                    bold: true,
                    italic: false,
                }],
                size: 16.0,
                gap_before: 16.0,
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "Body text".into(),
                    bold: false,
                    italic: false,
                }],
                size: 12.0,
                gap_before: 16.0,
            },
        ];
        let html = lines_to_html(&[page]);
        assert_eq!(
            html,
            "<h1><strong>Document title</strong></h1>\n<h2><strong>Section</strong></h2>\n<p>Body text</p>"
        );
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
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "Section".into(),
                    bold: true,
                    italic: false,
                }],
                size: 16.0,
                gap_before: 16.0,
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "Subsection".into(),
                    bold: true,
                    italic: false,
                }],
                size: 14.0,
                gap_before: 16.0,
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "Nested subsection".into(),
                    bold: false,
                    italic: true,
                }],
                size: 13.0,
                gap_before: 32.0,
            },
            StyledLine {
                runs: vec![StyledRun {
                    text: "Body".into(),
                    bold: false,
                    italic: false,
                }],
                size: 12.0,
                gap_before: 32.0,
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
