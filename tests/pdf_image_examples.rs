use kpapers_api::pdf_processing::{extract_pdf, repair_sponsr_title_from_filename};
use pdf_oxide::PdfDocument;

const MAX_PDF_BYTES: usize = 25 * 1024 * 1024;
const RUSSIAN_LIST_FILENAME: &str =
    "2026.11.05 192688 [Новостной деск] Апдейты по бумагам «Русского списка» [skladchik.org].pdf";
const CRIMEAN_BRIDGE_FILENAME: &str =
    "2026.11.19 195811 [Новостной деск] Последствия ударов по Крымскому мосту [skladchik.org].pdf";

fn extract_with_source_title(
    bytes: &[u8],
    filename: &str,
) -> kpapers_api::pdf_processing::ExtractedText {
    let mut extracted = extract_pdf(bytes, MAX_PDF_BYTES).expect("fixture PDF should extract");
    repair_sponsr_title_from_filename(filename, &mut extracted);
    extracted
}

fn manual_example_body(markdown: &str) -> &str {
    markdown
        .split_once("\n\n")
        .expect("manual example should begin with its review note")
        .1
        .trim()
}

#[test]
fn russian_list_page_13_vector_chart_is_rendered_and_embedded_between_text_blocks() {
    let bytes = include_bytes!("fixtures/pdf-image-extraction/2026-11-05-russian-list.pdf");
    let document = PdfDocument::from_bytes(bytes.to_vec()).expect("fixture PDF should open");
    let page_13_images = document
        .extract_images(12)
        .expect("page image extraction should succeed");
    let extracted = extract_with_source_title(bytes, RUSSIAN_LIST_FILENAME);

    assert_eq!(extracted.page_count, 27);
    assert!(
        page_13_images.is_empty(),
        "page 13 is a vector figure rather than an image XObject"
    );
    assert!(
        extracted
            .full_text
            .contains("Для простоты на графике три индекса"),
        "the text surrounding the page 13 chart should remain in the extraction"
    );
    let intro = extracted
        .html
        .find("Для простоты на графике три индекса")
        .expect("page 13 chart introduction should be present");
    let legend = extracted
        .html
        .find("желтый")
        .expect("page 13 chart legend should be present");
    let chart = &extracted.html[intro..legend];
    assert!(chart.contains("<img src=\"data:image/jpeg;base64,"));
    assert!(chart.contains("width=\"1033\" height=\"458\""));
    assert_eq!(
        extracted.markdown.trim(),
        manual_example_body(include_str!(
            "../docs/examples/pdf-image-extraction/2026-11-05-russian-list.md"
        ))
    );
}

#[test]
fn crimean_bridge_page_14_image_is_inline_after_its_intro_and_before_its_caption() {
    let bytes = include_bytes!("fixtures/pdf-image-extraction/2026-11-19-crimean-bridge.pdf");
    let document = PdfDocument::from_bytes(bytes.to_vec()).expect("fixture PDF should open");
    let page_14_images = document
        .extract_images(13)
        .expect("page image extraction should succeed");
    let extracted = extract_with_source_title(bytes, CRIMEAN_BRIDGE_FILENAME);

    assert_eq!(extracted.page_count, 23);
    assert_eq!(page_14_images.len(), 1);
    let long_title = extracted
        .html
        .find("контуры «плана Кушнера»")
        .expect("full article title should be present");
    assert!(
        !extracted.html[..long_title].contains("<img "),
        "images above the full article title are masthead branding and should be omitted"
    );

    let intro = extracted
        .html
        .find("Кстати, тут ваш покорный слуга натолкнулся на интересный инструмент")
        .expect("page 14 chart introduction should be present");
    let caption = extracted
        .html
        .find("TradingView: LSE.")
        .expect("page 14 chart caption should be present");
    assert!(intro < caption);
    let chart_block = &extracted.html[intro..caption];
    let image = chart_block
        .find("<img ")
        .map(|offset| &chart_block[offset..])
        .expect("page 14 chart should be embedded between its introduction and caption");
    assert!(image.starts_with("<img src=\"data:image/jpeg;base64,"));
    assert!(image.contains("width=\"1200\" height=\"533\""));

    let markdown_intro = extracted
        .markdown
        .find("Кстати, тут ваш покорный слуга натолкнулся на интересный инструмент")
        .expect("Markdown should include the chart introduction");
    let markdown_caption = extracted
        .markdown
        .find("TradingView: LSE.")
        .expect("Markdown should include the chart caption");
    assert!(markdown_intro < markdown_caption);
    assert!(
        extracted.markdown[markdown_intro..markdown_caption].contains("data:image/jpeg;base64,")
    );
    assert_eq!(
        extracted.markdown.trim(),
        manual_example_body(include_str!(
            "../docs/examples/pdf-image-extraction/2026-11-19-crimean-bridge.md"
        ))
    );
}
