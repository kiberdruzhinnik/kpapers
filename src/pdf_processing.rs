use base64::{Engine, engine::general_purpose::STANDARD};
use image::{DynamicImage, ImageEncoder, codecs::jpeg::JpegEncoder};
use pdf_oxide::PdfDocument;
use pdf_oxide::extractors::PdfImage;
use pdf_oxide::geometry::Rect;
use pdf_oxide::layout::TextLine;
use spellbook::Dictionary;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug)]
pub struct ExtractedText {
    pub full_text: String,
    pub html: String,
    pub markdown: String,
    pub first_page: String,
    pub page_count: usize,
}

const EXTRACTION_PIPELINE_REVISION: u32 = 37;
const MAX_PDF_IMAGE_COUNT: usize = 100;
const MAX_PDF_IMAGE_DIMENSION: u32 = 20_000;
const MAX_PDF_IMAGE_PIXELS: u64 = 8_000_000;
const MAX_PDF_TOTAL_IMAGE_PIXELS: u64 = 32_000_000;
const MAX_PDF_INLINE_IMAGE_BYTES: usize = 8 * 1024 * 1024;
const TRAILING_DISCLAIMER_SEARCH_PAGES: usize = 3;
const LITERAL_MARKDOWN_DISCLAIMER: &str = "**это не инвестиционные рекомендации**";
const INVESTMENT_DISCLAIMER_END: &str =
    "данный документ не является личной инвестиционной рекомендацией";
static VECTOR_RENDER_TEMP_ID: AtomicU64 = AtomicU64::new(0);
static RUSSIAN_DICTIONARY: OnceLock<Dictionary> = OnceLock::new();

const RUSSIAN_AFFIXES: &str = include_str!("../data/russian-hunspell/ru_RU.aff");
const RUSSIAN_WORDS: &str = include_str!("../data/russian-hunspell/ru_RU.dic");

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
    y: f32,
}

#[derive(Clone, Debug)]
struct PositionedImage {
    y: Option<f32>,
    html: String,
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
    let mut page_images = Vec::with_capacity(page_count);
    let mut image_count = 0usize;
    let mut total_image_pixels = 0u64;
    let mut total_inline_bytes = 0usize;
    for page_index in 0..page_count {
        let lines = document.extract_text_lines(page_index).map_err(|error| {
            format!(
                "PDF text extraction failed on page {}: {error}",
                page_index + 1
            )
        })?;
        pages.push(text_lines_to_styled_lines(lines));

        let mut positioned_images = Vec::new();
        match document.extract_images(page_index) {
            Ok(images) => {
                for (image_index, image) in images.into_iter().enumerate() {
                    if image_count >= MAX_PDF_IMAGE_COUNT {
                        tracing::warn!(
                            page = page_index + 1,
                            "PDF image count limit reached; later images are omitted"
                        );
                        break;
                    }
                    image_count += 1;
                    let width = image.width();
                    let height = image.height();
                    let pixels = u64::from(width).saturating_mul(u64::from(height));
                    if width == 0
                        || height == 0
                        || width > MAX_PDF_IMAGE_DIMENSION
                        || height > MAX_PDF_IMAGE_DIMENSION
                        || pixels > MAX_PDF_IMAGE_PIXELS
                        || total_image_pixels.saturating_add(pixels) > MAX_PDF_TOTAL_IMAGE_PIXELS
                    {
                        tracing::warn!(
                            page = page_index + 1,
                            image = image_index + 1,
                            width,
                            height,
                            "PDF image exceeds decoding limits; image omitted"
                        );
                        continue;
                    }
                    total_image_pixels = total_image_pixels.saturating_add(pixels);
                    match optimize_pdf_image(&image) {
                        Ok((uri, output_width, output_height)) => {
                            let Some(image_html) =
                                inline_jpeg_image_html(&uri, output_width, output_height)
                            else {
                                tracing::warn!(
                                    page = page_index + 1,
                                    image = image_index + 1,
                                    "PDF image did not match the safe inline markup policy; image omitted"
                                );
                                continue;
                            };
                            let inline_size = image_html.len();
                            if total_inline_bytes.saturating_add(inline_size)
                                > MAX_PDF_INLINE_IMAGE_BYTES
                            {
                                tracing::warn!(
                                    page = page_index + 1,
                                    image = image_index + 1,
                                    inline_size,
                                    "PDF inline image byte limit reached; image omitted"
                                );
                                continue;
                            }
                            total_inline_bytes += inline_size;
                            positioned_images.push(PositionedImage {
                                y: image.bbox().map(|bbox| bbox.y + bbox.height / 2.0),
                                html: image_html,
                            });
                        }
                        Err(error) => {
                            tracing::warn!(page = page_index + 1, image = image_index + 1, %error, "PDF image processing failed; image omitted")
                        }
                    }
                }
            }
            Err(error) => {
                tracing::warn!(page = page_index + 1, %error, "PDF page image extraction failed; continuing with text")
            }
        }
        if positioned_images.is_empty() {
            for (figure_index, (bbox, image)) in
                extract_vector_figures(bytes, &document, page_index, &pages[page_index])
                    .into_iter()
                    .enumerate()
            {
                if image_count >= MAX_PDF_IMAGE_COUNT {
                    tracing::warn!(
                        page = page_index + 1,
                        "PDF image count limit reached; later images are omitted"
                    );
                    break;
                }
                image_count += 1;
                let pixels = u64::from(image.width()).saturating_mul(u64::from(image.height()));
                if image.width() == 0
                    || image.height() == 0
                    || image.width() > MAX_PDF_IMAGE_DIMENSION
                    || image.height() > MAX_PDF_IMAGE_DIMENSION
                    || pixels > MAX_PDF_IMAGE_PIXELS
                    || total_image_pixels.saturating_add(pixels) > MAX_PDF_TOTAL_IMAGE_PIXELS
                {
                    tracing::warn!(
                        page = page_index + 1,
                        image = figure_index + 1,
                        "Rendered PDF figure exceeds decoding limits; image omitted"
                    );
                    continue;
                }
                total_image_pixels = total_image_pixels.saturating_add(pixels);
                match optimize_decoded_image(image) {
                    Ok((uri, width, height)) => {
                        let Some(image_html) = inline_jpeg_image_html(&uri, width, height) else {
                            continue;
                        };
                        let inline_size = image_html.len();
                        if total_inline_bytes.saturating_add(inline_size)
                            > MAX_PDF_INLINE_IMAGE_BYTES
                        {
                            tracing::warn!(
                                page = page_index + 1,
                                image = figure_index + 1,
                                inline_size,
                                "PDF inline image byte limit reached; rendered figure omitted"
                            );
                            continue;
                        }
                        total_inline_bytes += inline_size;
                        positioned_images.push(PositionedImage {
                            y: Some(bbox.y + bbox.height / 2.0),
                            html: image_html,
                        });
                    }
                    Err(error) => {
                        tracing::warn!(page = page_index + 1, image = figure_index + 1, %error, "PDF vector figure rendering failed; figure omitted")
                    }
                }
            }
        }
        page_images.push(positioned_images);
    }

    finalize_pages_with_images(pages, page_images, page_count)
}

fn optimize_pdf_image(image: &PdfImage) -> Result<(String, u32, u32), String> {
    let width = image.width();
    let height = image.height();
    let pixels = u64::from(width).saturating_mul(u64::from(height));
    if width == 0
        || height == 0
        || width > MAX_PDF_IMAGE_DIMENSION
        || height > MAX_PDF_IMAGE_DIMENSION
        || pixels > MAX_PDF_IMAGE_PIXELS
    {
        return Err("image dimensions exceed processing limits".into());
    }

    let decoded = image
        .to_dynamic_image()
        .map_err(|error| format!("image decoding failed: {error}"))?;
    optimize_decoded_image(decoded)
}

fn optimize_decoded_image(decoded: DynamicImage) -> Result<(String, u32, u32), String> {
    let width = decoded.width();
    let height = decoded.height();
    let rgba = decoded.to_rgba8();
    let mut rgb = image::RgbImage::new(width, height);
    for (x, y, pixel) in rgba.enumerate_pixels() {
        let alpha = u32::from(pixel[3]);
        let blend_white =
            |channel: u8| ((u32::from(channel) * alpha + 255 * (255 - alpha) + 127) / 255) as u8;
        rgb.put_pixel(
            x,
            y,
            image::Rgb([
                blend_white(pixel[0]),
                blend_white(pixel[1]),
                blend_white(pixel[2]),
            ]),
        );
    }

    let mut jpeg = Vec::new();
    JpegEncoder::new_with_quality(&mut jpeg, 95)
        .write_image(&rgb, width, height, image::ExtendedColorType::Rgb8)
        .map_err(|error| format!("JPEG intermediate encoding failed: {error}"))?;

    let mut parameters = caesium::parameters::CSParameters::new();
    parameters.jpeg.quality = 80;
    parameters.jpeg.preserve_icc = false;
    parameters.keep_metadata = false;
    parameters.keep_rotation = false;
    if width > 1200 {
        parameters.width = 1200;
    }
    let compressed = caesium::compress_in_memory(jpeg, &parameters)
        .map_err(|error| format!("libcaesium compression failed: {error}"))?;
    let output_width = width.min(1200);
    let output_height = if width > 1200 {
        ((u64::from(height) * 1200 + u64::from(width) / 2) / u64::from(width)) as u32
    } else {
        height
    };
    let data_uri = format!("data:image/jpeg;base64,{}", STANDARD.encode(compressed));
    Ok((data_uri, output_width, output_height))
}

fn extract_vector_figures(
    pdf_bytes: &[u8],
    document: &PdfDocument,
    page_index: usize,
    lines: &[StyledLine],
) -> Vec<(Rect, DynamicImage)> {
    let Ok(media) = document.get_page_media_box(page_index) else {
        return Vec::new();
    };
    let page_width = media.2 - media.0;
    let Ok(paths) = document.extract_paths(page_index) else {
        return Vec::new();
    };
    let mut figures = Vec::new();
    for path in paths {
        let bbox = path.rendered_bbox();
        if bbox.width < page_width * 0.45
            || bbox.height < 80.0
            || bbox.width * bbox.height < 10_000.0
            || bbox.x < media.0
            || bbox.y < media.1
            || bbox.x + bbox.width > media.2
            || bbox.y + bbox.height > media.3
        {
            continue;
        }

        // A figure is a large vector path isolated between two text lines.
        // Page-wide frames and paths overlapping text are therefore excluded.
        let nearest_line_above = lines
            .iter()
            .map(|line| line.y)
            .filter(|line_y| *line_y >= bbox.y + bbox.height)
            .min_by(f32::total_cmp);
        let nearest_line_below = lines
            .iter()
            .map(|line| line.y)
            .filter(|line_y| *line_y <= bbox.y)
            .max_by(f32::total_cmp);
        if !nearest_line_above.is_some_and(|line_y| line_y - (bbox.y + bbox.height) >= 2.0)
            || !nearest_line_below.is_some_and(|line_y| bbox.y - line_y >= 2.0)
        {
            continue;
        }

        let padding = 2.0;
        let crop_x = (bbox.x - padding).max(media.0);
        let crop_y = (bbox.y - padding).max(media.1);
        let crop_right = (bbox.x + bbox.width + padding).min(media.2);
        let crop_top = (bbox.y + bbox.height + padding).min(media.3);
        let crop = (crop_x, crop_y, crop_right, crop_top);
        if let Some(image) = render_pdf_region(pdf_bytes, page_index, media, crop) {
            figures.push((bbox, image));
        }
    }
    figures
}

fn render_pdf_region(
    pdf_bytes: &[u8],
    page_index: usize,
    media: (f32, f32, f32, f32),
    crop: (f32, f32, f32, f32),
) -> Option<DynamicImage> {
    const DPI: u32 = 150;
    let scale = DPI as f32 / 72.0;
    let page_width = media.2 - media.0;
    let page_height = media.3 - media.1;
    let output_pixels = (page_width * scale).ceil() * (page_height * scale).ceil();
    if !output_pixels.is_finite() || output_pixels > 4_000_000.0 {
        return None;
    }

    let id = VECTOR_RENDER_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    let temporary_dir =
        std::env::temp_dir().join(format!("kpapers-vector-{}-{id}", std::process::id()));
    std::fs::create_dir(&temporary_dir).ok()?;
    let output_prefix = temporary_dir.join("page");
    let output_path = output_prefix.with_extension("png");
    let page_number = (page_index + 1).to_string();
    let dpi = DPI.to_string();
    let child_result = Command::new("pdftoppm")
        .args([
            "-f",
            &page_number,
            "-l",
            &page_number,
            "-r",
            &dpi,
            "-png",
            "-singlefile",
            "-",
        ])
        .arg(&output_prefix)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    let Ok(mut child) = child_result else {
        let _ = std::fs::remove_dir_all(&temporary_dir);
        return None;
    };
    let write_result = child
        .stdin
        .take()
        .ok_or(std::io::ErrorKind::BrokenPipe.into())
        .and_then(|mut stdin| stdin.write_all(pdf_bytes));
    if write_result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
        let _ = std::fs::remove_dir_all(&temporary_dir);
        return None;
    }
    if !child.wait().ok()?.success() {
        let _ = std::fs::remove_dir_all(&temporary_dir);
        return None;
    }
    let full_page = image::open(&output_path).ok();
    let _ = std::fs::remove_dir_all(&temporary_dir);
    let full_page = full_page?;

    let x = ((crop.0 - media.0) * scale).round().max(0.0) as u32;
    let y = ((media.3 - crop.3) * scale).round().max(0.0) as u32;
    let width = ((crop.2 - crop.0) * scale).round().max(1.0) as u32;
    let height = ((crop.3 - crop.1) * scale).round().max(1.0) as u32;
    if x >= full_page.width() || y >= full_page.height() {
        return None;
    }
    let width = width.min(full_page.width() - x);
    let height = height.min(full_page.height() - y);
    Some(full_page.crop_imm(x, y, width, height))
}

fn inline_jpeg_image_html(data_uri: &str, width: u32, height: u32) -> Option<String> {
    let markup = format!(
        "<img src=\"{data_uri}\" alt=\"\" width=\"{width}\" height=\"{height}\" style=\"max-width:100%;height:auto\">"
    );
    is_safe_inline_jpeg_image_markup(&markup).then_some(markup)
}

fn is_safe_inline_jpeg_image_markup(markup: &str) -> bool {
    let Some(attributes) = markup.strip_prefix("<img src=\"data:image/jpeg;base64,") else {
        return false;
    };
    let Some((encoded, attributes)) = attributes.split_once("\" alt=\"\" width=\"") else {
        return false;
    };
    let Some((width, attributes)) = attributes.split_once("\" height=\"") else {
        return false;
    };
    let Some((height, attributes)) =
        attributes.split_once("\" style=\"max-width:100%;height:auto\">")
    else {
        return false;
    };
    if !attributes.is_empty() || encoded.is_empty() || encoded.len() % 4 != 0 {
        return false;
    }
    let mut padding = false;
    let mut padding_count = 0;
    for byte in encoded.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'+' | b'/' if !padding => {}
            b'=' => {
                padding = true;
                padding_count += 1;
                if padding_count > 2 {
                    return false;
                }
            }
            _ => return false,
        }
    }
    let Ok(width) = width.parse::<u32>() else {
        return false;
    };
    let Ok(height) = height.parse::<u32>() else {
        return false;
    };
    width > 0 && width <= 1200 && height > 0 && height <= MAX_PDF_IMAGE_DIMENSION
}

fn text_lines_to_styled_lines(lines: Vec<TextLine>) -> Vec<StyledLine> {
    let mut styled = Vec::with_capacity(lines.len());
    let mut previous_top = None;
    let mut quote_stack = Vec::new();
    for line in lines {
        let mut runs = Vec::new();
        let mut previous_word: Option<(&str, f32)> = None;
        let mut previous_word_style = None;
        let mut previous_word_closed_quote = false;
        for word in &line.words {
            let text = word.text.trim();
            if text.is_empty() {
                continue;
            }
            if let Some((previous_text, previous_right)) = previous_word
                && (should_insert_word_space(
                    previous_text,
                    text,
                    word.bbox.x - previous_right,
                    word.avg_font_size,
                ) || (previous_word_closed_quote
                    && text.chars().next().is_some_and(char::is_alphanumeric)))
            {
                let (previous_bold, previous_italic) = previous_word_style.unwrap_or_default();
                runs.push(StyledRun {
                    text: " ".into(),
                    bold: previous_bold && word.is_bold,
                    italic: previous_italic && word.is_italic,
                });
            }
            runs.push(StyledRun {
                text: text.into(),
                bold: word.is_bold,
                italic: word.is_italic,
            });
            previous_word_closed_quote = update_quote_stack(text, &mut quote_stack);
            previous_word = Some((text, word.bbox.x + word.bbox.width));
            previous_word_style = Some((word.is_bold, word.is_italic));
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
            y: line.bbox.y,
        });
    }
    styled
}

fn update_quote_stack(text: &str, quote_stack: &mut Vec<char>) -> bool {
    let mut closed_quote = false;
    for ch in text.chars() {
        if quote_stack.last() == Some(&ch) {
            quote_stack.pop();
            closed_quote = true;
        } else if let Some(close) = match ch {
            '„' => Some('“'),
            '«' => Some('»'),
            '“' => Some('”'),
            '‘' => Some('’'),
            _ => None,
        } {
            quote_stack.push(close);
        }
    }
    closed_quote
}

fn should_insert_word_space(previous: &str, next: &str, gap: f32, font_size: f32) -> bool {
    let Some(first_next) = next.chars().next() else {
        return false;
    };
    if matches!(
        first_next,
        ',' | '.' | ';' | ':' | '!' | '?' | '»' | '”' | '’' | ')' | ']' | '}'
    ) || previous.ends_with(['-', '‐', '‑'])
        || previous.ends_with(['«', '“', '‘', '(', '[', '{'])
        || matches!(first_next, '-' | '‐' | '‑')
    {
        return false;
    }

    // Opening parentheses begin parenthetical prose and are normally
    // separated from the preceding word even when PDF geometry is tight.
    if first_next == '(' {
        return true;
    }

    // Opening guillemets belong to the following quoted word, not the one
    // before them. Preserve the separating space even when PDF geometry is tight.
    if matches!(first_next, '«' | '„') {
        return true;
    }

    if crosses_latin_cyrillic_word_boundary(previous, next) {
        return true;
    }

    if crosses_letter_digit_boundary(previous, next) {
        return true;
    }

    gap > font_size.max(1.0) * 0.18
}

fn crosses_latin_cyrillic_word_boundary(previous: &str, next: &str) -> bool {
    let previous_char = previous
        .trim_end()
        .chars()
        .rev()
        .find(|ch| ch.is_alphanumeric());
    let next_char = next.chars().find(|ch| ch.is_alphanumeric());
    let (Some(previous_char), Some(next_char)) = (previous_char, next_char) else {
        return false;
    };

    (is_cyrillic(previous_char) && is_latin(next_char))
        || (is_latin(previous_char) && is_cyrillic(next_char))
}

fn crosses_letter_digit_boundary(previous: &str, next: &str) -> bool {
    let previous_char = previous
        .trim_end()
        .chars()
        .rev()
        .find(|ch| ch.is_alphanumeric());
    let next_char = next.chars().find(|ch| ch.is_alphanumeric());
    let (Some(previous_char), Some(next_char)) = (previous_char, next_char) else {
        return false;
    };

    (previous_char.is_alphabetic() && next_char.is_numeric())
        || (previous_char.is_numeric() && next_char.is_alphabetic())
}

fn is_latin(ch: char) -> bool {
    ch.is_ascii_alphabetic() || matches!(ch as u32, 0x00C0..=0x024F | 0x1E00..=0x1EFF)
}

fn is_cyrillic(ch: char) -> bool {
    matches!(ch as u32, 0x0400..=0x052F | 0x1C80..=0x1C8F | 0x2DE0..=0x2DFF | 0xA640..=0xA69F)
}

#[cfg(test)]
fn finalize_pages(pages: Vec<Vec<StyledLine>>, page_count: usize) -> Result<ExtractedText, String> {
    let page_images = vec![Vec::new(); pages.len()];
    finalize_pages_with_images(pages, page_images, page_count)
}

fn finalize_pages_with_images(
    mut pages: Vec<Vec<StyledLine>>,
    mut page_images: Vec<Vec<PositionedImage>>,
    page_count: usize,
) -> Result<ExtractedText, String> {
    page_images.resize_with(pages.len(), Vec::new);
    page_images.truncate(pages.len());
    if pages
        .last()
        .is_some_and(|lines| is_skladchik_boundary_ad(lines))
    {
        pages.pop();
        page_images.pop();
    }
    if pages
        .first()
        .is_some_and(|lines| is_skladchik_boundary_ad(lines))
    {
        pages.remove(0);
        page_images.remove(0);
    }
    strip_trailing_sponsr_navigation(&mut pages);
    page_images.truncate(pages.len());
    // Printed page numbering normally starts at 1 on the first content page, so
    // labels must be compared after leading advertisement pages are removed.
    strip_page_labels_by_content_position(&mut pages);
    if let Some(first_page) = pages.first_mut() {
        strip_sponsr_preamble(first_page);
        strip_crimsonanalytics_preamble(first_page);
    }
    strip_sponsr_branding(&mut pages);
    strip_images_above_first_content_line(&pages, &mut page_images);

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
    let html = lines_and_images_to_html(&pages, &page_images);
    let markdown = html_to_markdown(&html);
    Ok(ExtractedText {
        full_text,
        html,
        markdown,
        first_page,
        page_count,
    })
}

fn strip_images_above_first_content_line(
    pages: &[Vec<StyledLine>],
    page_images: &mut [Vec<PositionedImage>],
) {
    let (Some(first_page), Some(first_page_images)) = (pages.first(), page_images.first_mut())
    else {
        return;
    };
    let Some(first_content_y) = first_page
        .iter()
        .find(|line| !line.text().trim().is_empty())
        .map(|line| line.y)
    else {
        return;
    };
    first_page_images.retain(|image| image.y.is_none_or(|image_y| image_y <= first_content_y));
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
    let title = normalize_title_for_comparison(&title);
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
        let recovered_title = strip_html_tags(block);
        recovered_title.trim_start().starts_with("[Философское]")
            || normalize_title_for_comparison(&recovered_title)
                == normalize_title_for_comparison(&title)
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
    extracted.markdown = html_to_markdown(&extracted.html);
    extracted.first_page = format!(
        "{title}\n{}",
        remove_publication_timestamp(&extracted.first_page)
    );
    extracted.full_text = format!(
        "{title}\n\n{}",
        remove_publication_timestamp(&extracted.full_text)
    );
}

pub(crate) fn html_to_markdown(html: &str) -> String {
    let mut blocks = Vec::new();
    let mut quote_blocks = Vec::new();
    let mut in_quote = false;

    for raw_line in html.lines() {
        let mut line = raw_line.trim();
        if let Some(inner) = line.strip_prefix("<blockquote>") {
            in_quote = true;
            line = inner;
        }
        let closes_quote = in_quote && line.ends_with("</blockquote>");
        if closes_quote {
            line = line.strip_suffix("</blockquote>").unwrap_or(line);
        }
        let rendered = html_block_to_markdown(line);
        if !rendered.is_empty() {
            if in_quote {
                quote_blocks.push(rendered);
            } else {
                blocks.push(rendered);
            }
        }
        if closes_quote {
            if !quote_blocks.is_empty() {
                blocks.push(format!("> {}", quote_blocks.join("\n>\n> ")));
                quote_blocks.clear();
            }
            in_quote = false;
        }
    }

    if !quote_blocks.is_empty() {
        blocks.push(format!("> {}", quote_blocks.join("\n>\n> ")));
    }
    blocks.join("\n\n")
}

fn html_block_to_markdown(block: &str) -> String {
    for (tag, prefix) in [("h1", "# "), ("h2", "## "), ("h3", "### "), ("p", "")] {
        let open = format!("<{tag}>");
        let close = format!("</{tag}>");
        if let Some(content) = block
            .strip_prefix(&open)
            .and_then(|content| content.strip_suffix(&close))
        {
            return format!("{prefix}{}", html_inline_to_markdown(content));
        }
    }
    html_inline_to_markdown(block)
}

fn html_inline_to_markdown(input: &str) -> String {
    let mut output = String::new();
    let mut cursor = 0;
    while cursor < input.len() {
        let remaining = &input[cursor..];
        if remaining.starts_with("<img ")
            && let Some(tag_end) = remaining.find('>')
        {
            let tag_end = tag_end + 1;
            let tag = &remaining[..tag_end];
            if is_safe_inline_jpeg_image_markup(tag) {
                output.push_str(tag);
                cursor += tag_end;
                continue;
            }
        }
        let (token, replacement, is_formatting) = if remaining.starts_with("<strong>") {
            ("<strong>", Some("**"), true)
        } else if remaining.starts_with("</strong>") {
            ("</strong>", Some("**"), true)
        } else if remaining.starts_with("<em>") {
            ("<em>", Some("_"), true)
        } else if remaining.starts_with("</em>") {
            ("</em>", Some("_"), true)
        } else if remaining.starts_with("&amp;") {
            ("&amp;", Some("&"), false)
        } else if remaining.starts_with("&lt;") {
            ("&lt;", Some("<"), false)
        } else if remaining.starts_with("&gt;") {
            ("&gt;", Some(">"), false)
        } else {
            let ch = remaining.chars().next().expect("cursor is within input");
            push_markdown_text(&mut output, ch);
            cursor += ch.len_utf8();
            continue;
        };
        if let Some(replacement) = replacement {
            if is_formatting {
                output.push_str(replacement);
            } else {
                for ch in replacement.chars() {
                    push_markdown_text(&mut output, ch);
                }
            }
        }
        cursor += token.len();
    }
    output
}

fn push_markdown_text(output: &mut String, ch: char) {
    if matches!(ch, '\\' | '*' | '_' | '`' | '[' | ']' | '<' | '>' | '|') {
        output.push('\\');
    }
    output.push(ch);
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

fn normalize_title_for_comparison(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace("и\u{0306}", "й")
        .replace("И\u{0306}", "Й")
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

fn strip_trailing_sponsr_navigation(pages: &mut Vec<Vec<StyledLine>>) {
    let first_page_to_check = pages.len().saturating_sub(TRAILING_DISCLAIMER_SEARCH_PAGES);
    let mut tokens = Vec::new();
    for (page_index, page) in pages.iter().enumerate().skip(first_page_to_check) {
        for (line_index, line) in page.iter().enumerate() {
            for token in normalized_promotion_text(&line.text()).split_whitespace() {
                tokens.push((token.to_owned(), page_index, line_index));
            }
        }
    }

    let marker_tokens = INVESTMENT_DISCLAIMER_END
        .split_whitespace()
        .collect::<Vec<_>>();
    let marker_start = tokens.windows(marker_tokens.len()).rposition(|window| {
        window
            .iter()
            .map(|(token, _, _)| token.as_str())
            .eq(marker_tokens.iter().copied())
    });
    let Some(marker_start) = marker_start else {
        return;
    };
    let (_, page_index, line_index) = tokens[marker_start + marker_tokens.len() - 1];
    pages[page_index].truncate(line_index + 1);
    pages.truncate(page_index + 1);
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
    let previous_bold = previous.bold;
    let previous_italic = previous.italic;
    let next_has_space = next
        .and_then(|run| run.text.chars().next())
        .is_some_and(char::is_whitespace);
    let next_bold = next.is_some_and(|run| run.bold);
    let next_italic = next.is_some_and(|run| run.italic);
    let previous_text = target
        .iter()
        .map(|run| run.text.as_str())
        .collect::<String>();
    let next_text = next_runs
        .iter()
        .map(|run| run.text.as_str())
        .collect::<String>();
    if hyphenated_line_continuation(&previous_text, &next_text) {
        if discretionary_line_hyphen(&previous_text, &next_text) {
            remove_line_ending_hyphen_from_runs(target);
        }
        return;
    }
    if previous_has_space || next_has_space {
        return;
    }
    let bold = previous_bold && next_bold;
    let italic = previous_italic && next_italic;
    push_styled_char(target, ' ', bold, italic);
}

fn remove_line_ending_hyphen_from_runs(runs: &mut Vec<StyledRun>) {
    if let Some(last) = runs.last_mut() {
        let content_end = last.text.trim_end().len();
        last.text.truncate(content_end);
        if last
            .text
            .chars()
            .next_back()
            .is_some_and(|ch| matches!(ch, '-' | '‐' | '‑'))
        {
            last.text.pop();
        }
    }
    while runs.last().is_some_and(|run| run.text.is_empty()) {
        runs.pop();
    }
}

fn remove_line_ending_hyphen(text: &mut String) {
    let content_end = text.trim_end().len();
    text.truncate(content_end);
    if text
        .chars()
        .next_back()
        .is_some_and(|ch| matches!(ch, '-' | '‐' | '‑'))
    {
        text.pop();
    }
}

fn hyphenated_line_continuation(previous: &str, next: &str) -> bool {
    let previous = previous.trim_end();
    let next = next.trim_start();
    let ends_with_hyphen = previous
        .chars()
        .next_back()
        .is_some_and(|ch| matches!(ch, '-' | '‐' | '‑'));
    ends_with_hyphen
        && (next.chars().next().is_some_and(char::is_lowercase)
            || lexical_line_hyphen(previous, next))
}

fn lexical_line_hyphen(previous: &str, next: &str) -> bool {
    let previous = previous.trim_end();
    let next = next.trim_start();
    let Some(left) = previous
        .strip_suffix(['-', '‐', '‑'])
        .and_then(|word| word.rsplit(|ch: char| !ch.is_alphabetic()).next())
    else {
        return false;
    };
    let Some(right) = next.split(|ch: char| !ch.is_alphabetic()).next() else {
        return false;
    };
    if left.is_empty() || right.is_empty() || !is_russian_word(left) || !is_russian_word(right) {
        return false;
    }

    let joined = format!("{left}{right}").to_lowercase();
    let hyphenated = format!("{left}-{right}").to_lowercase();
    let dictionary = RUSSIAN_DICTIONARY.get_or_init(|| {
        Dictionary::new(RUSSIAN_AFFIXES, RUSSIAN_WORDS)
            .expect("bundled Russian Hunspell dictionary must parse")
    });
    let joined_is_word = dictionary.check(&joined);
    let hyphenated_is_word = dictionary.check(&hyphenated);

    // Only remove the hyphen when the lexicon supports the joined form and
    // rejects the hyphenated form. Ambiguous and unknown forms retain it.
    preserve_russian_hyphen(joined_is_word, hyphenated_is_word)
}

fn preserve_russian_hyphen(joined_is_word: bool, hyphenated_is_word: bool) -> bool {
    !joined_is_word || hyphenated_is_word
}

fn is_russian_word(word: &str) -> bool {
    !word.is_empty()
        && word
            .chars()
            .all(|ch| matches!(ch, 'а'..='я' | 'А'..='Я' | 'ё' | 'Ё'))
}

fn discretionary_line_hyphen(previous: &str, next: &str) -> bool {
    if !hyphenated_line_continuation(previous, next) {
        return false;
    }

    !lexical_line_hyphen(previous, next)
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

/// The first content line of a document has no preceding gap, so an opening
/// title is identified from its larger type and position instead of spacing.
fn is_opening_title_candidate(line: &StyledLine, body_size: f32, index: usize) -> bool {
    index == 0
        && (line.text().chars().count() <= 160
            || (line.text().trim_start().starts_with('[') && line.text().chars().count() <= 500))
        && line.size >= body_size * 1.35
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
    let continuation_char = if first_char == '(' {
        next_text
            .trim_start()
            .strip_prefix('(')
            .and_then(|text| text.trim_start().chars().next())
    } else {
        Some(first_char)
    };
    let hyphenated_continuation = hyphenated_line_continuation(&previous_text, &next_text);
    if (!continuation_char.is_some_and(char::is_lowercase) && !hyphenated_continuation)
        || is_heading_candidate(previous, body_size)
        || is_heading_candidate(next, body_size)
    {
        return false;
    }

    let similar_size = (previous.size - next.size).abs() <= previous.size.max(next.size) * 0.05;
    let matching_style = last_content_style(previous).is_some_and(|before| {
        first_content_style(next) == Some(before)
            || next
                .runs
                .iter()
                .filter(|run| !run.text.trim().is_empty())
                .any(|run| (run.bold, run.italic) == before)
    });
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
            if hyphenated_line_continuation(&previous.text(), &first.text()) {
                if discretionary_line_hyphen(&previous.text(), &first.text()) {
                    remove_line_ending_hyphen(&mut text);
                }
            } else {
                text.push_str(separator);
            }
        }
        if !text.is_empty() && previous_page_last.is_none() {
            text.push_str("\n\n");
        }
        for (index, line) in lines.iter().enumerate() {
            if index > 0 {
                let previous = lines[index - 1];
                if hyphenated_line_continuation(&previous.text(), &line.text()) {
                    if discretionary_line_hyphen(&previous.text(), &line.text()) {
                        remove_line_ending_hyphen(&mut text);
                    }
                } else if large_gap_starts_paragraph(previous, line) {
                    text.push_str("\n\n");
                } else {
                    text.push('\n');
                }
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
            let title_continuation = previous.text().trim_start().starts_with('[');
            let is_heading_sized = |line: &StyledLine| {
                (line.text().chars().count() <= 160
                    || (title_continuation && line.text().chars().count() <= 500))
                    && line.size >= body_size * 1.08
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

fn strip_crimsonanalytics_preamble(page: &mut Vec<StyledLine>) {
    let branding_end = (0..page.len().min(8)).find_map(|start| {
        (1..=3).find_map(|width| {
            let end = (start + width).min(page.len());
            let text = page[start..end]
                .iter()
                .map(StyledLine::text)
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase()
                .replace(['–', '—', '−'], "-");
            (text.contains("кримсон аналитика") && text.contains("crimsonanalytics")).then_some(end)
        })
    });
    let Some(branding_end) = branding_end else {
        return;
    };

    let Some(title_index) = page
        .iter()
        .enumerate()
        .skip(branding_end)
        .find_map(|(index, line)| {
            let text = line.text();
            (text.trim_start().starts_with('[') && text.chars().count() >= 24).then_some(index)
        })
    else {
        return;
    };
    let metadata = page[branding_end..title_index]
        .iter()
        .map(StyledLine::text)
        .collect::<Vec<_>>()
        .join(" ");
    let lower_metadata = metadata.to_lowercase();
    let has_reading_duration = lower_metadata
        .split_whitespace()
        .any(|part| part.parse::<u32>().is_ok())
        && ["минута", "минуты", "минут", "час", "часа", "часов"]
            .iter()
            .any(|unit| lower_metadata.contains(unit));
    if !has_publication_timestamp(&metadata) || !has_reading_duration {
        return;
    }

    page.drain(..title_index);
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

#[cfg(test)]
fn lines_to_html(pages: &[Vec<StyledLine>]) -> String {
    let page_images = vec![Vec::new(); pages.len()];
    lines_and_images_to_html(pages, &page_images)
}

fn lines_and_images_to_html(
    pages: &[Vec<StyledLine>],
    page_images: &[Vec<PositionedImage>],
) -> String {
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
        .filter(|(index, line)| {
            !is_literal_markdown_disclaimer(line)
                && (is_heading_candidate(line, body_size)
                    || is_opening_title_candidate(line, body_size, *index))
        })
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
    for (page_index, page) in pages.iter().enumerate() {
        let mut images = page_images.get(page_index).cloned().unwrap_or_default();
        images.sort_by(|left, right| match (left.y, right.y) {
            (Some(left_y), Some(right_y)) => right_y.total_cmp(&left_y),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
        let mut next_image = 0usize;
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
            for image in &images {
                blocks.push((
                    "img".to_owned(),
                    vec![StyledRun {
                        text: image.html.clone(),
                        bold: false,
                        italic: false,
                    }],
                    false,
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
            while images
                .get(next_image)
                .and_then(|image| image.y)
                .is_some_and(|image_y| image_y >= line.y)
            {
                if !paragraph.is_empty() {
                    blocks.push((
                        "p".to_owned(),
                        std::mem::take(&mut paragraph),
                        paragraph_quote,
                    ));
                }
                let image = &images[next_image];
                blocks.push((
                    "img".to_owned(),
                    vec![StyledRun {
                        text: image.html.clone(),
                        bold: false,
                        italic: false,
                    }],
                    false,
                ));
                next_image += 1;
            }
            if line.left >= base_left + 5.0 {
                quote_left = Some(line.left);
            } else if quote_left.is_some_and(|left| line.left < left - 2.0) {
                quote_left = None;
            }
            let in_quote = quote_left.is_some();
            let line_index = global_index;
            global_index += 1;
            if is_literal_markdown_disclaimer(line) {
                if !paragraph.is_empty() {
                    blocks.push((
                        "p".to_owned(),
                        std::mem::take(&mut paragraph),
                        paragraph_quote,
                    ));
                }
                let mut runs = line.runs.clone();
                for run in &mut runs {
                    run.bold = true;
                }
                blocks.push(("p".to_owned(), runs, in_quote));
                previous_line = Some(line);
                continue;
            }
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
                let hyphenated_continuation = previous_line.is_some_and(|previous| {
                    hyphenated_line_continuation(&previous.text(), &line.text())
                });
                let paragraph_break = previous_line
                    .is_some_and(|previous| large_gap_starts_paragraph(previous, line))
                    && !hyphenated_continuation;
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
        while next_image < images.len() {
            if !paragraph.is_empty() {
                blocks.push((
                    "p".to_owned(),
                    std::mem::take(&mut paragraph),
                    paragraph_quote,
                ));
            }
            let image = &images[next_image];
            blocks.push((
                "img".to_owned(),
                vec![StyledRun {
                    text: image.html.clone(),
                    bold: false,
                    italic: false,
                }],
                false,
            ));
            next_image += 1;
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
        if *tag == "img" {
            if let Some(image) = runs.first() {
                output.push(image.text.clone());
            }
            index += 1;
            continue;
        }
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

fn is_literal_markdown_disclaimer(line: &StyledLine) -> bool {
    line.text().trim() == LITERAL_MARKDOWN_DISCLAIMER
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
        PositionedImage, RUSSIAN_AFFIXES, RUSSIAN_DICTIONARY, RUSSIAN_WORDS, STANDARD, StyledLine,
        StyledRun, cache_fingerprint, discretionary_line_hyphen, extract_pdf, finalize_pages,
        html_to_markdown, inline_jpeg_image_html, is_safe_inline_jpeg_image_markup,
        is_skladchik_promotion, is_skladchik_purchase_promotion, is_xml_10_char,
        large_gap_continues_sentence, lexical_line_hyphen, lines_and_images_to_html, lines_to_html,
        normalize_text, optimize_pdf_image, pages_to_text, preserve_russian_hyphen,
        push_styled_char, strip_page_label_lines, strip_page_labels_by_content_position,
        strip_sponsr_branding, strip_sponsr_preamble, text_lines_to_styled_lines,
        text_to_safe_html,
    };
    use base64::Engine as _;
    use pdf_oxide::extractors::{ColorSpace, ImageData, PdfImage, PixelFormat};
    use pdf_oxide::geometry::Rect;
    use pdf_oxide::layout::{FontWeight, TextChar, TextLine, Word};

    use rand::{Rng, RngCore, SeedableRng, rngs::StdRng};
    use spellbook::Dictionary;

    const AD: &str = "skladchik.ORG\nТы еще\nне в Клубе?\nЧто такое Клуб «Складчик»?";

    #[test]
    fn positions_images_between_page_text_lines_by_vertical_bounds() {
        let mut earlier = line("Earlier", 12.0);
        earlier.y = 200.0;
        let mut later = line("Later", 12.0);
        later.y = 100.0;
        let pages = vec![vec![earlier, later]];
        let html = lines_and_images_to_html(
            &pages,
            &[vec![PositionedImage {
                y: Some(150.0),
                html: "<img src=\"data:image/jpeg;base64,between\" alt=\"\">".into(),
            }]],
        );
        assert!(
            html.find("Earlier").unwrap() < html.find("between").unwrap()
                && html.find("between").unwrap() < html.find("Later").unwrap(),
            "{html}"
        );
    }

    #[test]
    fn positions_images_without_bounds_after_their_page_text() {
        let pages = vec![vec![line("Page text", 12.0)]];
        let html = lines_and_images_to_html(
            &pages,
            &[vec![PositionedImage {
                y: None,
                html: "<img src=\"data:image/jpeg;base64,unpositioned\" alt=\"\">".into(),
            }]],
        );
        assert!(
            html.find("Page text").unwrap() < html.find("unpositioned").unwrap(),
            "{html}"
        );
    }

    #[test]
    fn optimizes_pdf_images_to_jpeg_at_or_below_the_width_limit() {
        let image = PdfImage::new(
            1201,
            2,
            ColorSpace::DeviceRGB,
            8,
            ImageData::Raw {
                pixels: vec![180; 1201 * 2 * 3],
                format: PixelFormat::RGB,
            },
        );
        let (uri, width, height) = optimize_pdf_image(&image).expect("image should compress");
        assert!(uri.starts_with("data:image/jpeg;base64,"));
        assert_eq!((width, height), (1200, 2));
        let encoded = uri.strip_prefix("data:image/jpeg;base64,").unwrap();
        let bytes = STANDARD.decode(encoded).expect("data URI should be base64");
        let decoded = image::load_from_memory(&bytes).expect("compressed image should decode");
        assert_eq!((decoded.width(), decoded.height()), (1200, 2));
        assert!(!bytes.windows(2).any(|marker| marker == [0xff, 0xe1]));
    }

    #[test]
    fn inline_image_policy_preserves_only_fixed_jpeg_data_uri_markup() {
        let data_uri = format!("data:image/jpeg;base64,{}", STANDARD.encode([1, 2, 3]));
        let markup = inline_jpeg_image_html(&data_uri, 3, 2).unwrap();
        assert!(is_safe_inline_jpeg_image_markup(&markup));
        assert!(inline_jpeg_image_html("https://example.test/image.jpg", 3, 2).is_none());
        assert!(inline_jpeg_image_html("data:image/png;base64,AQID", 3, 2).is_none());
        assert!(
            inline_jpeg_image_html("data:image/jpeg;base64,AQID\" onerror=\"alert(1)", 3, 2)
                .is_none()
        );
        assert!(inline_jpeg_image_html(&data_uri, 1201, 2).is_none());
    }

    #[test]
    fn markdown_preserves_safe_inline_images_in_article_order() {
        let data_uri = format!("data:image/jpeg;base64,{}", STANDARD.encode([1, 2, 3]));
        let image = inline_jpeg_image_html(&data_uri, 3, 2).unwrap();
        let markdown = html_to_markdown(&format!("<p>Before</p>\n{image}\n<p>After</p>"));
        assert!(markdown.find("Before").unwrap() < markdown.find(&image).unwrap());
        assert!(markdown.find(&image).unwrap() < markdown.find("After").unwrap());

        let unsafe_markdown = html_to_markdown(
            "<p>Before</p>\n<img src=\"https://example.test/image.jpg\">\n<p>After</p>",
        );
        assert!(unsafe_markdown.contains("\\<img src=\"https://"));
        assert!(unsafe_markdown.contains("Before") && unsafe_markdown.contains("After"));
    }

    #[test]
    fn rejects_oversized_pdf_images_before_decoding_pixel_data() {
        let image = PdfImage::new(
            super::MAX_PDF_IMAGE_DIMENSION + 1,
            1,
            ColorSpace::DeviceRGB,
            8,
            ImageData::Raw {
                pixels: Vec::new(),
                format: PixelFormat::RGB,
            },
        );
        assert!(optimize_pdf_image(&image).is_err());
    }

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
    fn removes_trailing_series_navigation_after_the_complete_disclaimer() {
        for (series, teaser) in [
            (
                "Кофе с банкиром",
                "[Новостной деск] Апдейты по бумагам Русского списка",
            ),
            (
                "Пропуск на новостной деск",
                "[Заметки] Испарение капитализации Google",
            ),
        ] {
            let pages = vec![
                vec![
                    line("Article closing paragraph remains.", 12.0),
                    line(
                        "Данная информация не является инвестиционной рекомендацией.",
                        12.0,
                    ),
                    line(
                        "Данный документ не является инвестиционной рекомендацией",
                        12.0,
                    ),
                    line("Данный документ не является личной", 12.0),
                    line("инвестиционной рекомендацией", 12.0),
                ],
                vec![line(series, 20.0), line(teaser, 14.0)],
                vec![
                    line("Продолжение карточки", 14.0),
                    line("05 нояб 15:53", 10.0),
                ],
            ];

            let extracted = finalize_pages(pages, 3).expect("article text should remain");
            assert!(
                extracted
                    .full_text
                    .contains("Article closing paragraph remains.")
            );
            assert!(
                extracted
                    .full_text
                    .contains("Данная информация не является")
            );
            assert!(
                extracted
                    .full_text
                    .contains("Данный документ не является инвестиционной рекомендацией")
            );
            assert!(
                extracted
                    .full_text
                    .contains("Данный документ не является личной инвестиционной рекомендацией")
            );
            assert!(!extracted.full_text.contains(series));
            assert!(!extracted.full_text.contains(teaser));
            assert!(!extracted.full_text.contains("Продолжение карточки"));
            assert!(!extracted.full_text.contains("05 нояб 15:53"));
            assert!(
                extracted
                    .html
                    .contains("Article closing paragraph remains.")
            );
            assert!(extracted.html.contains("Данная информация не является"));
            assert!(
                extracted
                    .html
                    .contains("Данный документ не является личной инвестиционной рекомендацией")
            );
            assert!(!extracted.html.contains(series));
            assert!(!extracted.html.contains(teaser));
            assert!(!extracted.html.contains("Продолжение карточки"));
            assert!(!extracted.html.contains("05 нояб 15:53"));
        }
    }

    #[test]
    fn preserves_trailing_text_when_the_disclaimer_end_marker_is_absent() {
        let extracted = finalize_pages(
            vec![vec![
                line("Article body remains.", 12.0),
                line("Related article navigation", 12.0),
            ]],
            1,
        )
        .expect("article text should remain");

        assert!(extracted.full_text.contains("Article body remains."));
        assert!(extracted.full_text.contains("Related article navigation"));
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
    fn removes_crimsonanalytics_header_before_the_article_title() {
        let pages = vec![vec![
            line("Кримсон Аналитика –", 22.0),
            line("CrimsonAnalytics", 22.0),
            line("Кримсон Аналитика – CrimsonAnalytics", 12.0),
            line("19 нояб 10:49", 10.0),
            line("1 минута", 10.0),
            line(
                "[Новостной деск] Последствия ударов по Крымскому мосту",
                22.0,
            ),
            line("Сразу к делу, дорогие коллеги.", 12.0),
        ]];

        let extracted = finalize_pages(pages, 1).expect("article text should remain");
        assert!(
            extracted
                .first_page
                .starts_with("[Новостной деск] Последствия ударов"),
            "{}",
            extracted.first_page
        );
        assert!(!extracted.first_page.contains("Кримсон Аналитика"));
        assert!(!extracted.first_page.contains("19 нояб 10:49"));
        assert!(!extracted.first_page.contains("1 минута"));
    }

    #[test]
    fn renders_long_wrapped_crimsonanalytics_headline_as_one_title() {
        let mut body_line = line("Тело статьи начинается здесь.", 12.0);
        body_line.gap_before = 30.0;
        let pages = vec![vec![
            line("Кримсон Аналитика – CrimsonAnalytics", 22.0),
            line("19 нояб 10:49", 10.0),
            line("1 минута", 10.0),
            line("[Новостной деск] Последствия ударов", 22.5),
            line("по Крымскому мосту, контуры «плана", 22.5),
            line("Кушнера» по продаже «украинского", 22.5),
            line("будущего» лондонским покровителям", 22.5),
            line("Зеленского и пять причин в пользу", 22.5),
            line("возобновления «горячей» войны с Ираном", 22.5),
            line("Сразу к делу, дорогие коллеги.", 12.0),
            body_line,
        ]];

        let mut extracted = finalize_pages(pages, 1).expect("article text should remain");
        super::repair_sponsr_title_from_filename(
            "[Новостнои\u{0306} деск] Последствия ударов по Крымскому мосту, контуры «плана Кушнера» по продаже «украинского будущего» лондонским покровителям Зеленского и пять причин в пользу возобновления «горячей» войны с Ираном [skladchik.org].pdf",
            &mut extracted,
        );
        let title = "[Новостной деск] Последствия ударов по Крымскому мосту, контуры «плана Кушнера» по продаже «украинского будущего» лондонским покровителям Зеленского и пять причин в пользу возобновления «горячей» войны с Ираном";
        assert!(
            extracted
                .html
                .starts_with(&format!("<h1><strong>{title}</strong></h1>")),
            "{}",
            extracted.html
        );
        assert_eq!(
            extracted.html.matches(title).count(),
            1,
            "{}",
            extracted.html
        );
        assert!(
            extracted
                .html
                .contains("<p>Сразу к делу, дорогие коллеги.</p>"),
            "{}",
            extracted.html
        );
        assert!(
            extracted
                .html
                .ends_with("<p>Тело статьи начинается здесь.</p>"),
            "{}",
            extracted.html
        );
        assert!(!extracted.html.contains("19 нояб 10:49"));
        assert!(!extracted.html.contains("1 минута"));
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
                    y: 0.0,
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
            y: 0.0,
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

    #[test]
    fn malformed_tounicode_cmap_does_not_panic() {
        let bytes = include_bytes!("../fuzz/corpus/pdf_bytes/cmap-invalid-utf8.pdf");

        assert!(extract_pdf(bytes, 2 * 1024 * 1024).is_err());
    }

    #[test]
    fn malformed_pdf_with_utf8_object_header_does_not_panic() {
        let bytes = include_bytes!("../fuzz/corpus/pdf_bytes/utf8_obj_boundary_crash");

        let _ = extract_pdf(bytes, 2 * 1024 * 1024);
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

    fn styled_pdf_page(lines: &[(&str, f32, f32)]) -> Vec<u8> {
        let mut stream = String::from("BT\n");
        for (text, size, y) in lines {
            let encoded = text
                .bytes()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            stream.push_str(&format!("1 0 0 1 50 {y} Tm /F1 {size} Tf <{encoded}> Tj\n"));
        }
        stream.push_str("ET");
        let objects = [
            "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>".to_owned(),
            format!(
                "<< /Length {} >>\nstream\n{stream}\nendstream",
                stream.len() + 1
            ),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_owned(),
        ];
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
    fn pdf_opening_title_and_lead_stay_separate_blocks_without_bold_title() {
        let extracted = super::extract_pdf(
            &styled_pdf_page(&[
                ("Ancient tribes share green", 22.5, 760.0),
                ("technologies", 22.5, 733.0),
                ("Today nothing matters more than kindness.", 15.0, 697.0),
                ("First body paragraph wraps", 15.0, 620.0),
                ("across two lines.", 15.0, 600.5),
                ("Second body paragraph follows.", 15.0, 540.0),
            ]),
            1024 * 1024,
        )
        .unwrap();
        assert_eq!(
            extracted.html,
            "<h1>Ancient tribes share green technologies</h1>\n<p>Today nothing matters more than kindness.</p>\n<p>First body paragraph wraps across two lines.</p>\n<p>Second body paragraph follows.</p>"
        );
    }

    #[test]
    fn keeps_wrapped_crimsonanalytics_title_and_bold_literal_disclaimer() {
        let mut disclaimer = line("", 12.0);
        disclaimer.runs = vec![
            StyledRun {
                text: "**это не ".into(),
                bold: false,
                italic: false,
            },
            StyledRun {
                text: "инвестиционные".into(),
                bold: true,
                italic: false,
            },
            StyledRun {
                text: " рекомендации**".into(),
                bold: false,
                italic: false,
            },
        ];
        let pages = vec![vec![
            line("[Кофе с банкиром] Последствия поражения", 20.0),
            line("Трампа и триумфа дипстейта: ИИ-активы,", 20.0),
            line("Ормузский кризис, золото и заработки на", 20.0),
            line("деклассированных элементах и спонсорах", 20.0),
            line("Демпартии.", 20.0),
            disclaimer,
            line("Сразу к делу, коллеги!", 12.0),
        ]];

        assert_eq!(
            lines_to_html(&pages),
            "<h1>[Кофе с банкиром] Последствия поражения Трампа и триумфа дипстейта: ИИ-активы, Ормузский кризис, золото и заработки на деклассированных элементах и спонсорах Демпартии.</h1>\n<p><strong>**это не инвестиционные рекомендации**</strong></p>\n<p>Сразу к делу, коллеги!</p>"
        );
    }

    #[test]
    fn preserves_unmatched_asterisks_as_ordinary_text() {
        assert_eq!(
            lines_to_html(&[vec![line("An unmatched ** marker remains literal.", 12.0)]]),
            "<p>An unmatched ** marker remains literal.</p>"
        );
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
    fn extraction_preserves_source_spelling_in_text_html_first_page_and_markdown() {
        let mut source = line("посвоему, мини-холлинг", 12.0);
        source.runs[0].bold = true;
        let extracted = finalize_pages(vec![vec![source]], 1).unwrap();
        let source_text = "посвоему, мини-холлинг";
        assert_eq!(extracted.full_text, source_text);
        assert_eq!(extracted.first_page, source_text);
        assert_eq!(
            extracted.html,
            format!("<p><strong>{source_text}</strong></p>")
        );
        assert_eq!(extracted.markdown, format!("**{source_text}**"));
    }

    #[test]
    fn line_end_hyphen_reconstruction_preserves_source_text() {
        let extracted = finalize_pages(
            vec![vec![
                line("Стоит начать жить по-", 12.0),
                line("своему, как это бывает", 12.0),
            ]],
            1,
        )
        .unwrap();
        assert!(!extracted.full_text.contains("[исправлено:"));
        assert!(!extracted.html.contains("[исправлено:"));
        assert!(!extracted.markdown.contains("[исправлено:"));
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
            markdown: String::new(),
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
        assert_eq!(
            extracted.markdown,
            "# **\\[Философское\\] «Старые деньги» — FAQ: как правильно считать дивиденды**\n\nСразу к делу, коллеги."
        );
    }

    #[test]
    fn filename_recovered_sponsr_title_keeps_body_blocks_and_clear_section_heading() {
        let mut extracted = super::ExtractedText {
            full_text: "27 февр 11:44 58 минут body".into(),
            html: "<p>27 февр 11:44</p>\n<h1><strong>это не инвестиционные рекомендации</strong></h1>\n<h2>Сразу к делу, коллеги.</h2>\n<h2><strong>Раздел статьи</strong></h2>\n<blockquote><p>Традиционные технические предуведомления:</p></blockquote>\n<p>Обычный текст после цитаты.</p>".into(),
            markdown: String::new(),
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
        assert!(extracted.markdown.starts_with("# **\\[Философское\\]"));
        assert!(
            extracted
                .markdown
                .contains("> Традиционные технические предуведомления:")
        );
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
    fn markdown_renderer_preserves_semantic_blocks_and_escapes_source_markup() {
        assert_eq!(
            super::html_to_markdown(
                "<h1>Title &amp; intro</h1>\n<h2><strong>Section</strong></h2>\n<p><em>Soft</em> and **literal** &lt;script&gt;</p>\n<blockquote><p>Notice one</p>\n<p><strong>Notice two</strong></p></blockquote>"
            ),
            "# Title & intro\n\n## **Section**\n\n_Soft_ and \\*\\*literal\\*\\* \\<script\\>\n\n> Notice one\n>\n> **Notice two**"
        );
    }

    #[test]
    fn cache_fingerprint_contains_pdf_oxide_and_processing_revision() {
        let source = "source-md5";
        let processed = cache_fingerprint(source);
        assert_eq!(processed, format!("pdf-text-v37:pdf_oxide:{source}"));
        assert_ne!(processed, format!("pdf-text-v35:pdf_oxide:{source}"));
        assert_ne!(processed, format!("pdf-text-v31:pdf_oxide:{source}"));
        assert_ne!(processed, format!("pdf-text-v29:pdf_oxide:{source}"));
        assert_ne!(processed, format!("pdf-text-v28:pdf_oxide:{source}"));
        assert_ne!(processed, format!("pdf-text-v27:pdf_oxide:{source}"));
        assert_ne!(processed, format!("pdf-text-v26:pdf_oxide:{source}"));
        assert_ne!(processed, format!("pdf-text-v25:pdf_oxide:{source}"));
        assert_ne!(processed, format!("pdf-text-v24:pdf_oxide:{source}"));
        assert_ne!(processed, format!("pdf-text-v23:pdf_oxide:{source}"));
        assert_ne!(processed, format!("pdf-text-v22:pdf_oxide:{source}"));
        assert_ne!(processed, format!("pdf-text-v20:pdf_oxide:{source}"));
        assert_ne!(processed, format!("pdf-text-v19:pdf_oxide:{source}"));
        assert_ne!(processed, format!("pdf-text-v18:pdf_oxide:{source}"));
        assert_ne!(processed, format!("pdf-text-v17:pdf_oxide:{source}"));
        assert_ne!(processed, format!("pdf-text-v16:pdf_oxide:{source}"));
        assert_ne!(processed, format!("pdf-text-v15:pdf_oxide:{source}"));
        assert_ne!(processed, format!("pdf-text-v14:pdf_oxide:{source}"));
        assert_ne!(processed, format!("pdf-text-v13:pdfium:{source}"));
        assert_ne!(processed, format!("pdf-text-v13:marker:{source}"));
        assert_ne!(processed, source);
        assert_eq!(cache_fingerprint(source), processed);
    }

    fn line_from_words(words: &[(&str, f32, bool)]) -> StyledLine {
        let mut positioned_words = Vec::new();
        let mut left = 0.0;
        for (text, gap_before, italic) in words {
            left += *gap_before;
            let mut chars = Vec::new();
            for (index, ch) in text.chars().enumerate() {
                let mut text_char = TextChar::simple(
                    ch,
                    Rect::new(left + index as f32 * 6.0, 20.0, 6.0, 12.0),
                    "Fixture".into(),
                    12.0,
                );
                text_char.is_italic = *italic;
                chars.push(text_char);
            }
            let word = Word::from_chars(chars);
            left += word.bbox.width;
            positioned_words.push(word);
        }
        text_lines_to_styled_lines(vec![TextLine::new(positioned_words)])
            .into_iter()
            .next()
            .expect("fixture has words")
    }

    #[test]
    fn reconstructs_spacing_before_parentheses_even_with_narrow_geometry() {
        let first = line_from_words(&[
            ("элите", 0.0, false),
            ("(если", 0.0, false),
            ("судить", 4.0, false),
            ("по", 4.0, false),
            ("заявлениям", 4.0, false),
            ("президента,", 4.0, false),
            ("например)", 4.0, false),
        ]);
        assert_eq!(
            first.text(),
            "элите (если судить по заявлениям президента, например)"
        );

        let second = line_from_words(&[
            ("руководства", 0.0, false),
            ("и", 4.0, false),
            ("(возможно)", 0.0, false),
            ("в головах", 4.0, false),
            ("политиков,", 4.0, false),
        ]);
        assert_eq!(
            second.text(),
            "руководства и (возможно) в головах политиков,"
        );

        let acronym_parenthetical = line_from_words(&[
            ("Tasnim,", 4.0, false),
            ("Корпус", 4.0, false),
            ("стражей", 4.0, false),
            ("исламской", 4.0, false),
            ("революции", 4.0, false),
            ("Ирана", 4.0, false),
            ("(КСИР)", 0.0, false),
            ("сообщил,", 4.0, false),
        ]);
        assert_eq!(
            acronym_parenthetical.text(),
            "Tasnim, Корпус стражей исламской революции Ирана (КСИР) сообщил,"
        );

        let saudi_oil_report = line_from_words(&[
            ("Саудовская", 4.0, false),
            ("Аравия", 4.0, false),
            ("сообщила", 4.0, false),
            ("Организации", 4.0, false),
            ("стран-экспортёров", 4.0, false),
            ("нефти", 4.0, false),
            ("(ОПЕК),", 0.0, false),
            ("что", 4.0, false),
            ("в", 4.0, false),
            ("октябре", 4.0, false),
            ("она", 4.0, false),
            ("добывала", 4.0, false),
            ("10,6", 4.0, false),
            ("млн", 4.0, false),
            ("баррелей", 4.0, false),
            ("нефти", 4.0, false),
            ("в", 4.0, false),
            ("сутки", 4.0, false),
            ("(б/с),", 0.0, false),
            ("по", 4.0, false),
            ("сравнению", 4.0, false),
            ("с", 4.0, false),
            ("8,372", 0.0, false),
            ("млн", 4.0, false),
            ("б/с", 4.0, false),
            ("в", 4.0, false),
            ("сентябре.", 4.0, false),
        ]);
        assert_eq!(
            saudi_oil_report.text(),
            "Саудовская Аравия сообщила Организации стран-экспортёров нефти (ОПЕК), что в октябре она добывала 10,6 млн баррелей нефти в сутки (б/с), по сравнению с 8,372 млн б/с в сентябре."
        );

        let reported = line_from_words(&[
            ("раз", 4.0, false),
            ("(", 0.0, false),
            ("с", 0.0, false),
            ("точки", 4.0, false),
            ("зрения", 4.0, false),
            ("её", 4.0, false),
            ("авторов", 4.0, false),
            (")", 0.0, false),
            ("находилась", 4.0, false),
        ]);
        assert_eq!(
            reported.text(),
            "раз (с точки зрения её авторов) находилась"
        );

        let second_reported = line_from_words(&[
            ("В", 4.0, false),
            ("этом", 4.0, false),
            ("смысле", 4.0, false),
            ("эффект", 4.0, false),
            ("наглядности", 4.0, false),
            ("должен", 4.0, false),
            ("был", 4.0, false),
            ("(", 0.0, false),
            ("и", 0.0, false),
            ("мог", 4.0, false),
            ("бы)", 4.0, false),
            ("сработать", 4.0, false),
        ]);
        assert_eq!(
            second_reported.text(),
            "В этом смысле эффект наглядности должен был (и мог бы) сработать"
        );

        let third_reported = line_from_words(&[
            ("горящие", 4.0, false),
            ("НПЗ", 4.0, false),
            ("и", 4.0, false),
            ("взорванные", 4.0, false),
            ("мосты", 4.0, false),
            ("—", 4.0, false),
            ("это", 4.0, false),
            ("очень", 4.0, false),
            ("круто", 4.0, false),
            ("(", 0.0, false),
            ("с", 0.0, false),
            ("точки", 4.0, false),
            ("зрения", 4.0, false),
            ("наших", 4.0, false),
            ("оппонентов)", 4.0, false),
            (",", 0.0, false),
            ("но", 4.0, false),
        ]);
        assert_eq!(
            third_reported.text(),
            "горящие НПЗ и взорванные мосты — это очень круто (с точки зрения наших оппонентов), но"
        );

        let punctuation = line_from_words(&[
            ("атаки,", 0.0, false),
            ("например", 4.0, false),
            (")", 0.0, false),
        ]);
        assert_eq!(punctuation.text(), "атаки, например)");

        let language_boundary = line_from_words(&[
            ("Посмотрите", 4.0, false),
            ("(", 0.0, false),
            ("это", 4.0, false),
            ("цитата", 4.0, false),
            ("из", 4.0, false),
            ("Rаpporteur,", 0.0, false),
            ("т.", 4.0, false),
            ("е.", 4.0, false),
            ("самый", 4.0, false),
            ("мейнстримный", 4.0, false),
        ]);
        assert_eq!(
            language_boundary.text(),
            "Посмотрите (это цитата из Rаpporteur, т. е. самый мейнстримный"
        );

        let quoted_boundary = line_from_words(&[
            ("[Прим.", 4.0, false),
            ("Кримсон:", 4.0, false),
            ("под", 4.0, false),
            ("„", 4.0, false),
            ("миром", 0.0, false),
            ("“", 0.0, false),
            ("подразумевается", 0.0, false),
            ("„", 4.0, false),
            ("мир", 0.0, false),
            ("“", 0.0, false),
            ("на", 0.0, false),
            ("лондских", 4.0, false),
            ("условиях],", 4.0, false),
        ]);
        assert_eq!(
            quoted_boundary.text(),
            "[Прим. Кримсон: под „миром“ подразумевается „мир“ на лондских условиях],"
        );

        let quote_before_word = line_from_words(&[
            ("а", 4.0, false),
            ("другой", 4.0, false),
            ("стороне", 4.0, false),
            ("—", 4.0, false),
            ("„ехать“", 4.0, false),
            ("из", 0.0, false),
            ("известного", 4.0, false),
            ("советского", 4.0, false),
            ("анекдота». ", 4.0, false),
        ]);
        assert_eq!(
            quote_before_word.text(),
            "а другой стороне — „ехать“ из известного советского анекдота»."
        );

        let quote_before_words_in_report = line_from_words(&[
            ("Тегеран", 4.0, false),
            ("разрабатывает", 4.0, false),
            ("планы", 4.0, false),
            ("по", 4.0, false),
            ("возможному", 4.0, false),
            ("подрыву", 4.0, false),
            ("„грязной", 4.0, false),
            ("бомбы“", 4.0, false),
            ("в", 0.0, false),
            ("столице", 4.0, false),
            ("США.", 4.0, false),
            ("„Грязная", 4.0, false),
            ("бомба“", 4.0, false),
            ("использует", 0.0, false),
            ("обычные", 4.0, false),
            ("взрывчатые", 4.0, false),
            ("вещества", 4.0, false),
            ("для", 4.0, false),
            ("рассеивания", 4.0, false),
            ("радиоактивных", 4.0, false),
            ("материалов.", 4.0, false),
        ]);
        assert_eq!(
            quote_before_words_in_report.text(),
            "Тегеран разрабатывает планы по возможному подрыву „грязной бомбы“ в столице США. „Грязная бомба“ использует обычные взрывчатые вещества для рассеивания радиоактивных материалов."
        );

        let opening_guillemet = line_from_words(&[
            ("или", 4.0, false),
            ("готовность", 4.0, false),
            ("к", 4.0, false),
            ("отступлению", 4.0, false),
            ("и", 4.0, false),
            ("вообще", 4.0, false),
            ("в", 4.0, false),
            ("целом", 4.0, false),
            ("как", 4.0, false),
            ("«сопливость»,", 0.0, false),
            ("а", 4.0, false),
            ("не", 4.0, false),
            ("благородство", 4.0, false),
            ("или", 4.0, false),
            ("гуманизм.", 4.0, false),
        ]);
        assert_eq!(
            opening_guillemet.text(),
            "или готовность к отступлению и вообще в целом как «сопливость», а не благородство или гуманизм."
        );

        let opening_guillemet_after_preposition = line_from_words(&[
            ("Вот", 4.0, false),
            ("это", 4.0, false),
            ("получается", 4.0, false),
            ("про", 4.0, false),
            ("«суть». ", 0.0, false),
        ]);
        assert_eq!(
            opening_guillemet_after_preposition.text(),
            "Вот это получается про «суть»."
        );

        let opening_guillemet_after_preposition_in_phrase = line_from_words(&[
            ("идущие", 4.0, false),
            ("по", 4.0, false),
            ("«несанкционированным»", 0.0, false),
            ("маршрутам,", 4.0, false),
        ]);
        assert_eq!(
            opening_guillemet_after_preposition_in_phrase.text(),
            "идущие по «несанкционированным» маршрутам,"
        );

        let opening_low_quote = line_from_words(&[
            ("и", 4.0, false),
            ("кто", 4.0, false),
            ("будет", 4.0, false),
            ("платить", 4.0, false),
            ("за", 4.0, false),
            ("„фонды", 0.0, false),
            ("поддержки“", 4.0, false),
            ("новопринятого", 0.0, false),
            ("члена,", 4.0, false),
        ]);
        assert_eq!(
            opening_low_quote.text(),
            "и кто будет платить за „фонды поддержки“ новопринятого члена,"
        );

        let month_year_boundary = line_from_words(&[
            ("Его", 4.0, false),
            ("запустили", 4.0, false),
            ("в", 4.0, false),
            ("марте", 4.0, false),
            ("2026", 0.0, false),
            ("года,", 4.0, false),
        ]);
        assert_eq!(
            month_year_boundary.text(),
            "Его запустили в марте 2026 года,"
        );

        let war_date_boundary = line_from_words(&[
            ("С", 4.0, false),
            ("начала", 4.0, false),
            ("войны", 4.0, false),
            ("28", 0.0, false),
            ("февраля,", 4.0, false),
            ("когда", 4.0, false),
            ("США", 4.0, false),
            ("и", 4.0, false),
            ("Израиль", 4.0, false),
            ("нанесли", 4.0, false),
            ("удары,", 4.0, false),
        ]);
        assert_eq!(
            war_date_boundary.text(),
            "С начала войны 28 февраля, когда США и Израиль нанесли удары,"
        );

        let wildberries = line_from_words(&[
            ("инфраструктуру,", 4.0, false),
            ("такую", 4.0, false),
            ("как", 4.0, false),
            ("Wildberries,", 0.0, false),
            ("на", 4.0, false),
            ("самом", 4.0, false),
            ("деле,", 4.0, false),
            ("вероятно,", 4.0, false),
        ]);
        assert_eq!(
            wildberries.text(),
            "инфраструктуру, такую как Wildberries, на самом деле, вероятно,"
        );

        let number_boundary = line_from_words(&[
            ("Причина,", 4.0, false),
            ("по", 4.0, false),
            ("которой", 4.0, false),
            ("высокопоставленные", 4.0, false),
            ("чиновники", 4.0, false),
            ("Пентагона", 4.0, false),
            ("только", 4.0, false),
            ("после", 4.0, false),
            ("4", 0.0, false),
            ("(!)", 0.0, false),
            ("лет", 4.0, false),
            ("СВО", 4.0, false),
            ("открыли", 4.0, false),
        ]);
        assert_eq!(
            number_boundary.text(),
            "Причина, по которой высокопоставленные чиновники Пентагона только после 4 (!) лет СВО открыли"
        );
    }

    #[test]
    fn reconstructs_reported_russian_spacing_and_preserves_italic_quote() {
        let first = line_from_words(&[
            ("У", 4.0, false),
            ("современной", 4.0, false),
            ("поп", 4.0, false),
            ("-", 0.0, false),
            ("психологии", 0.0, false),
            ("на", 4.0, false),
            ("эту", 4.0, false),
            ("тему", 4.0, false),
            ("имеется", 4.0, false),
            ("сколько", 4.0, false),
            ("угодно", 4.0, false),
            ("воркования", 4.0, false),
            ("вида", 4.0, false),
            ("«надо", 4.0, false),
            ("понять", 4.0, false),
            ("и", 4.0, false),
            ("учесть»,", 4.0, false),
            ("«инерция", 4.0, false),
            ("восприятия»,", 4.0, false),
            ("«привычка", 4.0, false),
            ("к", 4.0, false),
            ("прежнему", 4.0, false),
            ("распределению", 4.0, false),
            ("ролей", 4.0, false),
            ("спаситель", 4.0, true),
            ("—", 4.0, true),
            ("заколдованная", 4.0, true),
            ("жертва", 4.0, true),
            ("»", 0.0, true),
            ("и", 4.0, false),
            ("всё", 4.0, false),
            ("такое.", 4.0, false),
        ]);
        let text = first.text();
        assert_eq!(
            text,
            "У современной поп-психологии на эту тему имеется сколько угодно воркования вида «надо понять и учесть», «инерция восприятия», «привычка к прежнему распределению ролей спаситель — заколдованная жертва» и всё такое."
        );
        let html = lines_to_html(&[vec![first]]);
        assert_eq!(
            html,
            "<p>У современной поп-психологии на эту тему имеется сколько угодно воркования вида «надо понять и учесть», «инерция восприятия», «привычка к прежнему распределению ролей <em>спаситель — заколдованная жертва»</em> и всё такое.</p>"
        );

        let second = line_from_words(&[
            ("Звучит", 4.0, false),
            ("абсолютно", 4.0, false),
            ("по", 4.0, false),
            ("-", 0.0, false),
            ("идиотски,", 0.0, false),
            ("бессмысленно,", 4.0, false),
            ("беспощадно,", 4.0, false),
            ("ужасно", 4.0, false),
            ("и", 4.0, false),
            ("отвратительно.", 4.0, false),
            ("Но", 4.0, false),
            ("именно", 4.0, false),
            ("этим", 4.0, false),
            ("занимались", 4.0, false),
            ("примитивные", 4.0, false),
            ("племена", 4.0, false),
            ("Австралии", 4.0, false),
            ("до", 4.0, false),
            ("самого", 4.0, false),
            ("прихода", 4.0, false),
            ("европейцев", 4.0, false),
            ("(и,", 4.0, false),
            ("возможно,", 4.0, false),
            ("мы", 4.0, false),
            ("не", 4.0, false),
            ("погрешим", 4.0, false),
            ("против", 4.0, false),
            ("истины,", 4.0, false),
            ("предположив,", 4.0, false),
            ("что", 4.0, false),
            ("не", 4.0, false),
            ("в", 4.0, false),
            ("последнюю", 4.0, false),
            ("очередь", 4.0, false),
            ("благодаря", 4.0, false),
            ("этому", 4.0, false),
            ("идиотизму", 4.0, false),
            ("европейцы", 4.0, false),
            ("застали", 4.0, false),
            ("аборигенов", 4.0, false),
            ("в", 4.0, false),
            ("худшем,", 4.0, false),
            ("кажется,", 4.0, false),
            ("культурно", 4.0, false),
            ("-", 0.0, false),
            ("техническом", 0.0, false),
            ("состоянии,", 4.0, false),
            ("чем", 4.0, false),
            ("кого", 4.0, false),
            ("бы", 4.0, false),
            ("то", 4.0, false),
            ("ни", 4.0, false),
            ("было", 4.0, false),
            ("из", 4.0, false),
            ("встретившихся", 4.0, false),
            ("им", 4.0, false),
            ("дикарей).", 4.0, false),
        ]);
        assert_eq!(
            second.text(),
            "Звучит абсолютно по-идиотски, бессмысленно, беспощадно, ужасно и отвратительно. Но именно этим занимались примитивные племена Австралии до самого прихода европейцев (и, возможно, мы не погрешим против истины, предположив, что не в последнюю очередь благодаря этому идиотизму европейцы застали аборигенов в худшем, кажется, культурно-техническом состоянии, чем кого бы то ни было из встретившихся им дикарей)."
        );
        assert_eq!(
            lines_to_html(&[vec![second]]),
            "<p>Звучит абсолютно по-идиотски, бессмысленно, беспощадно, ужасно и отвратительно. Но именно этим занимались примитивные племена Австралии до самого прихода европейцев (и, возможно, мы не погрешим против истины, предположив, что не в последнюю очередь благодаря этому идиотизму европейцы застали аборигенов в худшем, кажется, культурно-техническом состоянии, чем кого бы то ни было из встретившихся им дикарей).</p>"
        );
    }

    #[test]
    fn preserves_bold_word_metadata_through_html_rendering() {
        let make_word = |text: &str, left: f32, weight: FontWeight| {
            let chars = text
                .chars()
                .enumerate()
                .map(|(index, ch)| {
                    let mut text_char = TextChar::simple(
                        ch,
                        Rect::new(left + index as f32 * 6.0, 20.0, 6.0, 12.0),
                        "Fixture".into(),
                        12.0,
                    );
                    text_char.font_weight = weight;
                    text_char
                })
                .collect();
            Word::from_chars(chars)
        };
        let words = vec![
            make_word("Слово", 0.0, FontWeight::Normal),
            make_word("важно", 42.0, FontWeight::Bold),
            make_word("здесь.", 84.0, FontWeight::Normal),
        ];
        let lines = text_lines_to_styled_lines(vec![TextLine::new(words)]);
        assert_eq!(lines[0].runs[1].text, " ");
        assert!(!lines[0].runs[1].bold);
        assert_eq!(lines[0].runs[2].text, "важно");
        assert!(lines[0].runs[2].bold);

        let html = lines_to_html(&[lines]);
        assert_eq!(html, "<p>Слово <strong>важно</strong> здесь.</p>");
    }

    #[test]
    fn word_separator_uses_layout_for_spaces_and_text_for_punctuation() {
        assert_eq!(
            line_from_words(&[("два", 0.0, false), ("слова", 3.0, false)]).text(),
            "два слова"
        );
        assert_eq!(
            line_from_words(&[("слово", 8.0, false), (",", 0.0, false), ("»", 0.0, false)]).text(),
            "слово,»"
        );
        assert_eq!(
            line_from_words(&[
                ("по", 8.0, false),
                ("-", 8.0, false),
                ("идиотски", 0.0, false)
            ])
            .text(),
            "по-идиотски"
        );
        assert_eq!(
            line_from_words(&[("«", 8.0, false), ("слово", 0.0, false)]).text(),
            "«слово"
        );
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
    fn preserves_compounds_split_across_lines_and_ordinary_wrapped_spaces() {
        let pages = vec![vec![
            line(
                "В конце спича рассказчик предложил добавить ему и совместно, по-",
                12.0,
            ),
            line("товарищески, приобрести и употребить беленькую.", 12.0),
            line("То есть, как бы героя ни кидало по жизни, чем-", 12.0),
            line(
                "нибудь нужным он занимался неизменно — в любую погоду.",
                12.0,
            ),
            line("В качестве объяснения-", 12.0),
            line("мастерключа напечатана формула.", 12.0),
            line("который ему обещали проповедники-", 12.0),
            line("фундаменталисты (диспенсационалисты).", 12.0),
            line("Обычный перенос строки", 12.0),
            line("должен сохранить пробел.", 12.0),
        ]];
        let expected = "В конце спича рассказчик предложил добавить ему и совместно, по-товарищески, приобрести и употребить беленькую. То есть, как бы героя ни кидало по жизни, чем-нибудь нужным он занимался неизменно — в любую погоду. В качестве объяснения-мастерключа напечатана формула. который ему обещали проповедники-фундаменталисты (диспенсационалисты). Обычный перенос строки должен сохранить пробел.";

        assert_eq!(normalize_text(&pages_to_text(&pages)), expected);
        assert_eq!(lines_to_html(&pages), format!("<p>{expected}</p>"));
    }

    #[test]
    fn preserves_reported_lexical_hyphens_across_same_page_and_page_boundary_wraps() {
        let compounds = [
            ("во", "вторых", "во-вторых"),
            ("какой", "либо", "какой-либо"),
            ("государств", "членов", "государств-членов"),
            ("Украина", "Россия", "Украина-Россия"),
            ("Косиняк", "Камыш", "Косиняк-Камыш"),
            ("Эр", "Рияде", "Эр-Рияде"),
        ];

        for (left, right, compound) in compounds {
            let first_line = format!("В тексте {left}-");
            let second_line = format!("{right} встречается далее.");
            let expected = format!("В тексте {compound} встречается далее.");

            let same_page = vec![vec![line(&first_line, 12.0), line(&second_line, 12.0)]];
            assert_eq!(normalize_text(&pages_to_text(&same_page)), expected);
            assert_eq!(lines_to_html(&same_page), format!("<p>{expected}</p>"));

            let page_boundary = vec![
                vec![line(&first_line, 12.0)],
                vec![line(&second_line, 12.0)],
            ];
            assert_eq!(normalize_text(&pages_to_text(&page_boundary)), expected);
            assert_eq!(lines_to_html(&page_boundary), format!("<p>{expected}</p>"));
        }
    }

    #[test]
    fn russian_lexicon_classifies_only_confirmed_discretionary_hyphens() {
        let dictionary = RUSSIAN_DICTIONARY.get_or_init(|| {
            Dictionary::new(RUSSIAN_AFFIXES, RUSSIAN_WORDS)
                .expect("bundled Russian Hunspell dictionary must parse")
        });
        assert!(dictionary.check("пристрастились"));
        assert!(!dictionary.check("пристрасти-лись"));
        assert!(discretionary_line_hyphen("пристрасти-", "лись"));
        assert!(dictionary.check("достопримечательностями"));
        assert!(discretionary_line_hyphen("достопримечатель-", "ностями"));

        // The joined spelling is not in the lexicon, so preserve the boundary
        // hyphen instead of corrupting an unknown word.
        assert!(!dictionary.check("вовторых"));
        assert!(!discretionary_line_hyphen("во-", "вторых"));
        assert!(lexical_line_hyphen("неизвестнослово-", "неизвестнаячасть"));
        assert!(discretionary_line_hyphen("frozen-", "thawed"));
        assert!(discretionary_line_hyphen("Rаpporteur-", "other"));

        assert!(!preserve_russian_hyphen(true, false));
        assert!(preserve_russian_hyphen(false, true));
        assert!(preserve_russian_hyphen(true, true));
        assert!(preserve_russian_hyphen(false, false));
    }

    #[test]
    fn removes_discretionary_hyphens_from_reported_line_endings_in_text_and_html() {
        let extracted = finalize_pages(
            vec![vec![
                line_from_words(&[("пристрасти", 0.0, false), ("-", 0.0, false)]),
                line_from_words(&[
                    ("лись", 0.0, false),
                    ("к", 4.0, false),
                    ("портикам,", 4.0, false),
                    ("термам", 4.0, false),
                    ("и", 4.0, false),
                    ("изысканным", 4.0, false),
                    ("пиршествам.", 4.0, false),
                ]),
                line_from_words(&[("неискушен", 0.0, false), ("-", 0.0, false)]),
                line_from_words(&[
                    ("ными", 0.0, false),
                    ("и", 4.0, false),
                    ("простодушными,", 4.0, false),
                    ("образованностью", 4.0, false),
                    ("и", 4.0, false),
                    ("просвещённостью.", 4.0, false),
                ]),
            ]],
            1,
        )
        .unwrap();

        for text in ["пристрастились", "неискушенными"] {
            assert!(
                extracted.full_text.contains(text),
                "{}",
                extracted.full_text
            );
            assert!(extracted.html.contains(text), "{}", extracted.html);
        }
        assert!(!extracted.full_text.contains("пристрасти-лись"));
        assert!(!extracted.full_text.contains("неискушен-ными"));
    }

    #[test]
    fn joins_the_reported_explanation_wrap_in_final_extracted_text_and_html() {
        // Captured line split from the supplied PDF: page 138 ends the line after "объяснения-".
        let extracted = finalize_pages(
            vec![vec![
                line("в качестве объяснения-", 12.0),
                line("мастерключа её следующая формула:", 12.0),
                line("Обычная строка продолжается", 12.0),
                line("следующим словом.", 12.0),
            ]],
            1,
        )
        .unwrap();

        assert!(
            extracted
                .full_text
                .contains("объяснения-мастерключа её следующая формула:")
        );
        assert!(
            extracted
                .html
                .contains("объяснения-мастерключа её следующая формула:")
        );
        assert!(
            extracted
                .full_text
                .contains("Обычная строка продолжается следующим словом.")
        );
        assert!(
            extracted
                .html
                .contains("Обычная строка продолжается следующим словом.")
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
    fn joins_page_continuation_when_first_word_is_emphasized() {
        let pages = vec![
            vec![line("уже отвечалось, потому что", 15.0)],
            vec![StyledLine {
                runs: vec![
                    StyledRun {
                        text: "любой".into(),
                        bold: false,
                        italic: true,
                    },
                    StyledRun {
                        text: " вопрос есть вопрос добра и зла.".into(),
                        bold: false,
                        italic: false,
                    },
                ],
                size: 15.0,
                gap_before: 0.0,
                left: 50.0,
                y: 0.0,
            }],
        ];
        let expected = "уже отвечалось, потому что любой вопрос есть вопрос добра и зла.";

        assert_eq!(normalize_text(&pages_to_text(&pages)), expected);
        assert_eq!(
            lines_to_html(&pages),
            "<p>уже отвечалось, потому что <em>любой</em> вопрос есть вопрос добра и зла.</p>"
        );
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
    fn joins_parenthetical_continuation_across_pages_but_keeps_completed_boundary() {
        let continuation_pages = vec![
            vec![line(
                "Это по идее не служение, а самообслуживание — и на практике",
                12.0,
            )],
            vec![line(
                "(просто напоминаю статистику) данный лайфстайл приводит к росту тревог.",
                12.0,
            )],
        ];
        let expected = "Это по идее не служение, а самообслуживание — и на практике (просто напоминаю статистику) данный лайфстайл приводит к росту тревог.";
        assert_eq!(
            normalize_text(&pages_to_text(&continuation_pages)),
            expected
        );
        assert_eq!(
            lines_to_html(&continuation_pages),
            format!("<p>{expected}</p>")
        );

        let completed_pages = vec![
            vec![line("Первый абзац полностью завершен.", 12.0)],
            vec![line("(Это отдельное пояснение.)", 12.0)],
        ];
        assert_eq!(
            normalize_text(&pages_to_text(&completed_pages)),
            "Первый абзац полностью завершен.\n\n(Это отдельное пояснение.)"
        );
        assert_eq!(
            lines_to_html(&completed_pages),
            "<p>Первый абзац полностью завершен.</p>\n<p>(Это отдельное пояснение.)</p>"
        );
    }

    #[test]
    fn joins_reported_parenthetical_page_continuation_in_text_and_html() {
        let first_page = "Осознанность&Саморазвитие же суть россыпь полутерапевтических- полуколдуских (соотношение может сильно варьироваться) практик, чья финальная задача, она же высшая ценность, состоит в «чувствовать себя хорошо». Это по идее не служение, а самообслуживание — и на практике";
        let second_page = "(просто напоминаю статистику) данный лайфстайл приводит не к снижению, а к росту тревог и случаев душевных расстройств.";
        let pages = vec![vec![line(first_page, 12.0)], vec![line(second_page, 12.0)]];
        let expected = format!("{first_page} {second_page}");

        assert_eq!(normalize_text(&pages_to_text(&pages)), expected);
        assert_eq!(
            lines_to_html(&pages),
            format!("<p>{}</p>", expected.replace('&', "&amp;"))
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
                y: 0.0,
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
                y: 0.0,
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
                y: 0.0,
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
                y: 0.0,
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
                y: 0.0,
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
                y: 0.0,
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
                y: 0.0,
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
                y: 0.0,
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
                y: 0.0,
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
                y: 0.0,
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
                y: 0.0,
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
                y: 0.0,
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
                y: 0.0,
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
                y: 0.0,
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
                y: 0.0,
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
                y: 0.0,
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
            y: 0.0,
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
            y: 0.0,
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
                y: 0.0,
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
                y: 0.0,
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
                y: 0.0,
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
                y: 0.0,
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
                y: 0.0,
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
