# Tasks

## 1. Add the PDF reference fixture

- [x] 1.1 Copy the supplied PDF unchanged to `tests/fixtures/pdf-bold-formatting/` and verify the fixture opens and reports 10 pages through `pdf_oxide::PdfDocument`.

## 2. Recognize synthetic bold rendering

- [x] 2.1 Infer bold weight for filled-and-stroked text with a positive stroke width no greater than 4% of font size, while preserving font-metadata bold and leaving larger outlines unclassified; verify the rule with focused `pdf_oxide` tests.

## 3. Verify reference PDF output

- [x] 3.1 Update `tests/pdf_bold_formatting.rs` to verify the selected page 2 and page 3 bold passages and page 8 bold-italic phrase in HTML and Markdown, and verify ordinary text remains unwrapped.
- [x] 3.2 Verify extracted plain text still contains the selected passages and run `cargo test --test pdf_bold_formatting` successfully.
