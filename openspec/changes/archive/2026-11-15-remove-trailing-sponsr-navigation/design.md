# Design

## Context

See proposal.md for the motivation and specs/pdf-ingestion/spec.md for the behavior contract. `extract_pdf` gathers positioned lines per page, then `finalize_pages` applies cleanup before deriving text and HTML. The supplied Sponsr PDFs retain a legal disclaimer followed by series navigation and a related-article teaser; the teaser may continue onto another page. The final full-page Skladchik advertisement is already handled separately. For spacing, `text_lines_to_styled_lines` prevents spaces around hyphens between words on one visual line, but line assembly inserts a separator between adjacent visual lines, which can split a word broken after its hyphen.

## Goals / Non-Goals

**Goals:**
- Preserve the entire standard investment-disclaimer block.
- Remove the subsequent trailing navigation and related-content material across page boundaries.
- Keep cleanup fail-safe when the expected disclaimer ending is absent.
- Preserve hyphenated compounds when a visual line break separates the hyphen from the remainder of a word.
- Ensure cached documents are reprocessed after the extraction behavior changes.

**Non-Goals:**
- Detect or remove related-content cards in the middle of an article.
- Generalize cleanup to other publishers or changing disclaimer formats.
- Change article classification, title recovery, or full-page advertisement recognition.

## Decisions

- Apply the cleanup to page-aware extracted lines in `finalize_pages`, after existing boundary advertisement removal and before `first_page`, `full_text`, and HTML are derived. This keeps text and HTML consistent and allows the existing full-page ad rule to remain independent.
- Search only the trailing content region for the standard final disclaimer statement, `Данный документ не является личной инвестиционной рекомендацией`. Preserve the entire disclaimer section through that statement, then discard all subsequent extracted lines and pages. This closing statement follows the long legal disclaimer and the general investment-recommendation statement in both supplied PDFs, so it preserves all unmarked disclaimer text while removing the crossed navigation, teaser, and date. Restricting the search to the end avoids truncating an article that mentions the disclaimer in its body.
- If the closing statement is not found in the trailing region, leave the document unchanged. A changed or incomplete publisher format must not cause speculative content removal; the cleanup can be revisited when the format changes.
- During line joining for both extracted text and HTML, omit the normal line separator when the preceding line ends in a recognized hyphen (`-`, `‐`, or `‑`) and the next line begins with a lowercase continuation. Keep the separator for ordinary line breaks and do not treat en or em dashes as word-joining hyphens. The existing within-line word spacing behavior remains intact. In addition to `по-товарищески` and `чем-нибудь`, add a regression for the reported PDF wrap `объяснения-` / `мастерключа её следующая формула:` through the relevant extraction path. Use a minimized PDF fixture or captured extracted-line structure that reproduces the source line break, and assert joined extracted text and HTML; retain an ordinary line-break control.
- Add focused regression coverage for text and HTML consistency, preservation of all disclaimer statements and preceding article content, teaser continuation across a page boundary, and preservation when the terminal marker is absent. Use minimal representative line fixtures rather than embedding the full supplied articles.
- Advance `EXTRACTION_PIPELINE_REVISION` from 18 to 19 so existing cached output is regenerated on the next extraction without a database migration.

## Risks / Trade-offs

- [The disclaimer wording or order changes] → Require the known closing statement near the document end and preserve all content when it is absent; the user will request a rule update if the publisher format changes.
- [A real article ends with that exact disclaimer statement] → This is the agreed publisher convention; keep the match scoped to trailing pages and test that earlier article content remains intact.
- [The marker is split across extracted text runs or lines] → Normalize whitespace and compare normalized line text across the trailing region rather than relying on a single PDF text run.
- [A line-ending hyphen is punctuation rather than part of a split word] → Join directly only when the following line begins with a lowercase continuation and the preceding line ends in a recognized hyphen; preserve spaces for other punctuation and line breaks.

## Migration Plan

No database migration is required. Incrementing the processing revision to v19 makes cached v18 content miss and be rebuilt during the next normal extraction. If either cleanup rule needs to be withdrawn, restore the corresponding prior processing behavior and revision so content is regenerated using the prior pipeline.
