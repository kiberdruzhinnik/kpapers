# Design

## Context

See proposal.md for the reported extraction errors. The PDF processor joins wrapped lines through shared hyphen-continuation helpers: lowercase continuations are considered discretionary unless explicitly recognized as lexical compounds, while uppercase continuations do not enter that join path. The behavior affects both same-page line joining and page-boundary joining. Processed output is keyed by the extraction pipeline revision.

## Goals / Non-Goals

**Goals:**

- Preserve the five reported lexical compounds across visual line and page boundaries without adding whitespace around their hyphens.
- Keep existing discretionary word-break removal behavior.
- Apply the same lexical decision in extracted text and generated HTML.
- Ensure cached PDFs are reprocessed after the output behavior changes.

**Non-Goals:**

- Add a general-purpose Russian dictionary or infer arbitrary compounds beyond the reported lexical cases and existing recognized compounds.
- Change paragraph detection, source PDF text, or unrelated word-spacing rules.

## Decisions

1. **Distinguish recognized lexical compounds from discretionary breaks using compound context, not capitalization alone.** Extend the current lexical recognition so it can preserve the reported lowercase compounds (`какой-либо`, `государств-членов`) and uppercase compounds (`Украина-Россия`, `Косиняк-Камыш`, `Эр-Рияде`). A casing-only rule is insufficient because lowercase line endings can be either lexical or discretionary, and uppercase continuations are not always a compound. A full language dictionary is out of scope.

2. **Use one decision consistently for same-page and page-boundary joins.** The line and page text paths must agree about whether to keep the hyphen, remove a discretionary hyphen, or insert a separator. Duplicating separate lexical rules risks the same compound being rendered differently in extracted text and HTML.

3. **Invalidate cached extraction output when the pipeline changes.** Increment the extraction pipeline revision so existing cached PDFs are processed again and receive corrected text.

## Risks / Trade-offs

- [A recognized compound may be mistaken for discretionary hyphenation or vice versa] -> Add regression cases for every reported lexical compound and existing discretionary examples.
- [Different join paths may produce inconsistent text and HTML] -> Exercise both same-page and cross-page cases through the text and HTML output paths.

## Migration Plan

No data migration is required. The pipeline revision change causes cached PDFs to be reprocessed on their next refresh. If the change regresses output, revert the join behavior and restore the prior revision.
