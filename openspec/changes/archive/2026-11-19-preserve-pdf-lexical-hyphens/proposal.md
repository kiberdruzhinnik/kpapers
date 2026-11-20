# Proposal

## Why

PDF extraction can corrupt lexical compounds when a visual line or page break falls after their hyphen. Lowercase continuations may be treated as discretionary hyphenation and lose the hyphen, while uppercase continuations can be joined with an unwanted space.

## What Changes

- Preserve lexical hyphens across visual line and page boundaries for the reported compounds: `какой-либо`, `государств-членов`, `Украина-Россия`, `Косиняк-Камыш`, and `Эр-Рияде`.
- Continue removing discretionary line-ending hyphens from words split across lines, and preserve existing paragraph boundaries and spacing between distinct words.
- Add regression coverage for the reported compounds in extracted text and published HTML.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pdf-ingestion`: Preserve lexical hyphens in the reported lower- and uppercase compounds when PDF line or page wrapping separates their components.

## Impact

Changes affect PDF text reconstruction and focused regression coverage in `src/pdf_processing.rs`. The PDF processing revision and cache identity may need an update so previously cached documents are reprocessed. No API or dependency changes are expected.
