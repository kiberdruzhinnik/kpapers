use kpapers_api::pdf_processing::extract_pdf;
use pdf_oxide::PdfDocument;

const MAX_PDF_BYTES: usize = 25 * 1024 * 1024;
const REFERENCE_PDF: &[u8] =
    include_bytes!("fixtures/pdf-bold-formatting/dark-enlighteners-feudalism-again.pdf");

#[test]
fn preserves_bold_and_bold_italic_formatting_from_reference_pdf() {
    let document =
        PdfDocument::from_bytes(REFERENCE_PDF.to_vec()).expect("reference PDF should open");
    assert_eq!(
        document
            .page_count()
            .expect("page count should be readable"),
        10
    );

    let extracted =
        extract_pdf(REFERENCE_PDF, MAX_PDF_BYTES).expect("reference PDF text should extract");

    let page_two_bold = "В случае мискастинга фильм проваливается в прокате, а история заканчивается не новой эпохой, а пробуксовыванием или фейлом. Или вообще коллапсом";
    let page_three_heading = "Миром правят сами знаете кто";
    let page_eight_bold_italic = "Это философия, стратегия и даже метафизика";
    let normal_text = "Сегодня для нас нет ничего актуальней философий.";

    for passage in [page_two_bold, page_three_heading, page_eight_bold_italic] {
        assert!(
            extracted.full_text.contains(passage),
            "plain extracted text should contain {passage:?}"
        );
    }

    assert!(
        extracted
            .html
            .contains(&format!("<strong>{page_two_bold}</strong>")),
        "HTML should preserve the bold page 2 passage: {}",
        extracted.html
    );
    assert!(
        extracted
            .html
            .contains(&format!("<strong>{page_three_heading}</strong>")),
        "HTML should preserve the bold page 3 heading: {}",
        extracted.html
    );
    let bold_italic_html = format!("<strong>{page_eight_bold_italic}</strong></em>");
    let bold_italic_html_start = extracted
        .html
        .find(&bold_italic_html)
        .expect("HTML should preserve bold inside the page 8 italic quotation");
    assert!(
        extracted.html[..bold_italic_html_start]
            .rfind("<em>")
            .is_some_and(
                |italic_start| extracted.html[italic_start..bold_italic_html_start]
                    .find("</em>")
                    .is_none()
            ),
        "HTML should keep the bold phrase inside its italic quotation"
    );

    assert!(
        extracted.markdown.contains(&format!("**{page_two_bold}**")),
        "Markdown should preserve the bold page 2 passage"
    );
    assert!(
        extracted
            .markdown
            .contains(&format!("**{page_three_heading}**")),
        "Markdown should preserve the bold page 3 heading"
    );
    assert!(
        extracted
            .markdown
            .contains(&format!("**{page_eight_bold_italic}**")),
        "Markdown should preserve bold inside the page 8 italic quotation"
    );
    assert!(
        extracted.full_text.contains(normal_text),
        "plain extracted text should retain ordinary text"
    );
    assert!(
        !extracted
            .html
            .contains(&format!("<strong>{normal_text}</strong>")),
        "HTML should not classify ordinary text as bold"
    );
    assert!(
        !extracted.markdown.contains(&format!("**{normal_text}**")),
        "Markdown should not classify ordinary text as bold"
    );
}
