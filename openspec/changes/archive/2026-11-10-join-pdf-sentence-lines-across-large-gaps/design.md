# Design

## Context

See proposal.md for motivation and `specs/pdf-ingestion/spec.md` for the behavior contract. PDFium-derived `StyledLine` values retain text, font size, style runs, vertical gap, and left position. HTML paragraph grouping currently starts a new paragraph when a within-page gap exceeds 1.8 times the line size. The sample PDF has a 39-point gap between a line ending in a comma and its same-aligned, same-style continuation.

## Goals / Non-Goals

**Goals:**
- Prevent a large gap from overriding strong evidence that prose continues the same sentence.
- Keep large gaps effective as paragraph-boundary evidence in ordinary cases.
- Apply one consistent paragraph grouping decision to published HTML and extracted plain text.

**Non-Goals:**
- General Russian or multilingual grammatical parsing.
- Reclassifying headings, notices, or blockquotes.
- Preserving the attached PDF as a repository fixture; synthetic line fixtures can exercise the measured layout conditions without adding the source document.

## Decisions

### Override large-gap paragraph breaks only with multiple continuation cues

When a large gap would normally start a paragraph, consider joining the adjacent lines only when several high-confidence cues agree: the preceding line ends with continuation punctuation (initially comma, semicolon, or colon), the next line begins with a lowercase letter, their left positions are compatible with the same text block, and their dominant font size and inline style are compatible. Retain the current gap-based break when these cues do not agree. Keep the punctuation set and thresholds small and explicit so behavior remains reviewable.

Alternative considered: remove the vertical-gap paragraph rule. This would fix the supplied sentence but would merge unrelated paragraphs throughout documents. Full NLP or language-specific sentence parsing was also considered disproportionate and less reliable for the multilingual PDF corpus.

### Centralize paragraph-boundary decisions

Use a shared boundary decision for the HTML formatter and plain-text extraction so both outputs agree about whether adjacent lines belong to one paragraph. Preserve page-boundary continuation handling and the current explicit empty-line behavior unless the same continuation evidence applies. Heading and notice classification remain separate concerns.

Alternative considered: adjust only the HTML threshold. The current plain-text path collapses line breaks differently from HTML, so a separate HTML-only tweak risks keeping the published feed and cached text inconsistent.

### Test the sample layout without checking in the source PDF

Add a focused synthetic line-level regression with the sample's structure: matching size/style/alignment, a 39-point gap, a comma-ending first line, and a lowercase continuation. Add counterexamples for a completed sentence followed by a large-gap paragraph and for incompatible layout/style. If a future user-provided or distributable fixture becomes available, it can supplement these focused tests.

Alternative considered: commit the attached PDF. It is unnecessary for validating the grouping decision and may introduce a copyrighted external document into the repository.

## Risks / Trade-offs

- [Continuation punctuation and lowercase starts can occur at real paragraph boundaries] -> Require compatible layout and styling as corroborating evidence and add counterexample tests.
- [The same heuristic may not fit every writing system] -> Keep the rule conservative and avoid claiming general linguistic sentence understanding.
- [A shared grouping helper could alter cached plain text] -> Cover both HTML and text output in regression cases and increment the extraction cache revision if observable cached output changes.

## Migration Plan

No schema or provider migration is expected. If the implementation changes cached extracted text or HTML, increment the PDF extraction pipeline revision so existing cached entries are regenerated. Rollback is restoring the prior application version, which reuses the prior revision and formatting behavior.
