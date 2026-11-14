# Design

## Context

See `proposal.md` for motivation. The current adapter receives word text and bounding boxes from `pdf_oxide`, but inserts a literal space before every word after the first. Those runs then feed plain text and styled HTML generation.

## Goals / Non-Goals

**Goals:**
- Reconstruct separators from extracted word text and horizontal layout.
- Keep style boundaries independent from separator decisions so spaces and punctuation remain correct across italic and other style changes.
- Ensure corrected output replaces cached output through the existing processing revision mechanism.

**Non-Goals:**
- Change PDF engine selection, OCR behavior, paragraph detection, or heading classification.
- Apply language-specific spelling correction to extracted content.

## Decisions

- Use adjacent word content and bounding-box geometry to decide whether a separator is needed. Text cues must attach closing punctuation (including commas and closing guillemets) to preceding content and keep hyphenated compounds joined; geometry distinguishes actual gaps between words. Alternatives such as joining every word or relying only on punctuation rules cannot preserve ordinary spaces reliably.
- Keep the separator as a styled run or attach it to a neighboring run without changing the word's bold/italic flags. This preserves existing HTML emphasis behavior while avoiding style changes from affecting spacing.
- Cover the reported article phrases in regression cases for both normalized text and HTML, and include ordinary inter-word spaces to guard against over-joining.
- Increment the extraction pipeline revision when the behavior is implemented so unchanged source PDFs are reprocessed from cache.

## Risks / Trade-offs

- [PDF text positions can vary with font metrics and kerning] → Use a size-relative gap threshold and exercise representative punctuation, compounds, and ordinary spaces in fixtures.
- [Punctuation may be emitted as a separate word or attached to a neighboring word] → Test both forms, including punctuation across style boundaries.

## Migration Plan

No storage migration is needed. Increment the processing revision so the next refresh regenerates cached PDF text and HTML. Reverting the code and revision restores the prior cache identity behavior.
