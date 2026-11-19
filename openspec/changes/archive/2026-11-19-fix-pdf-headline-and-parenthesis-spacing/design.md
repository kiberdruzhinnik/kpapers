# Design

## Context

See `proposal.md` for the reported output. `text_lines_to_styled_lines` currently inserts spaces from a measured gap threshold and exempts closing punctuation, but needs explicit handling for prose boundaries where narrow PDF geometry loses spaces: before opening parentheses and guillemets, after closing quotes, and between Latin/Cyrillic or letter/digit transitions. First-page cleanup recognizes the `sponsr` masthead and removes only complete CrimsonAnalytics branding lines. `merge_wrapped_title_lines` and heading detection both impose a 160-character per-line/title-candidate limit.

## Goals / Non-Goals

**Goals:**

- Keep PDF word reconstruction typography-aware while recovering conventional prose spacing before parenthetical phrases and opening guillemets, after closing quotes, and across Latin/Cyrillic and letter/digit boundaries.
- Preserve lexical hyphens when known compounds wrap between visual lines.
- Remove the recognizable CrimsonAnalytics header and its date/time and reading duration before the headline.
- Keep a long wrapped opening headline together and identify it as the document title.
- Preserve existing paragraph, quote, and safe-HTML behavior.

**Non-Goals:**

- Generalize publication-header parsing to unrelated publishers.
- Infer article titles from arbitrary filenames or promote ordinary body prose to headings.
- Change PDF download, cache storage, or public APIs.

## Decisions

1. **Recover conventional separators from word-boundary context.** Treat opening parentheses and guillemets as separators before the following phrase, and insert a separator after a recognized closing quote when the next token begins a word. Preserve separators between Latin and Cyrillic text and between letters and digits even when geometry reports little or no gap. Keep closing punctuation adjacent as today. A purely global whitespace normalization pass was rejected because it would lack word-boundary and punctuation context.

2. **Preserve established lexical compounds across visual line wraps.** Remove line-ending hyphens only when they are discretionary; retain known lexical hyphens such as `проповедники-фундаменталисты` when the compound is split between lines.

3. **Recognize the known CrimsonAnalytics first-page header as a unit.** Match the branding even when extraction splits it over styled lines, then use the following date/time and reading-duration metadata plus the identifiable headline to delimit the preamble. Keep the recognition constrained to this publisher signature rather than deleting generic dates or short opening lines. This avoids disturbing ordinary articles that happen to mention the brand or contain dates.

4. **Join wrapped title lines based on opening-title context, not a 160-character title cap.** Preserve compatible typography and adjacency checks when merging lines, but permit the merged title to exceed 160 characters when the recognized publication header and opening position establish it as the headline. Let this recognized opening title become `<h1>` without relaxing heading thresholds for ordinary prose elsewhere. Retaining the fixed cap was rejected because the supplied title itself exceeds it.

5. **Cover each behavior with focused line-level regression cases.** Add synthetic styled-word cases for missing separators before parentheses and guillemets, after closing quotes, across Latin/Cyrillic and letter/digit boundaries, for punctuation adjacency, and for lexical hyphens retained across line wraps. Include first-page/title cases for split branding, metadata removal, the complete long headline, separate lead, and preserved body. Use the supplied PDF as a manual or local regression input if fixtures can reference it without committing private source material.

## Risks / Trade-offs

- [Word-boundary recovery could add spaces in fragments or around punctuation] -> Limit inserted spaces to the explicit punctuation, script, and letter/digit boundaries and retain adjacency regression cases.
- [Publisher-header matching could remove real article text if too permissive] -> Require the recognizable branding and publication metadata sequence before removing the prefix.
- [Long-title recognition could promote an opening paragraph] -> Limit the exception to a visually supported title following the recognized header and preserve the existing ordinary-prose heading protections.

## Migration Plan

No data migration is expected. Bump the PDF extraction pipeline revision so cached processed output is regenerated under the corrected rules. If the behavior causes regressions, revert the extraction change and restore the prior revision behavior.
