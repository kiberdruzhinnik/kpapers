# Design

## Context

See `proposal.md` for motivation and `specs/pdf-ingestion/spec.md` for the observable requirements. `extract_pdf` currently finalizes page layout, derives `full_text`, `first_page`, HTML, and Markdown, then the extraction endpoint caches the text and HTML. RSS and Markdown exports consume that cached HTML. PDF cache identity already includes a processing revision.

## Goals / Non-Goals

**Goals:**

- Add a deterministic, local correction stage to the existing PDF processing pipeline.
- Preserve layout, emphasis, and block structure while annotating each applied correction in the visible text.
- Require empirical precision evidence before enabling any correction class.

**Non-Goals:**

- Send document text to cloud proofreading or language model services.
- Rewrite grammar, style, or author voice.
- Correct words outside Russian or English.

## Decisions

1. **Run corrections after extraction cleanup and before text/HTML/Markdown are finalized.** This lets spelling logic see reconstructed words rather than PDF word fragments, and keeps all cached representations aligned. Exact reviewed corrections split by a discretionary line ending hyphen are reconstructed before the exact-form rule runs. The implementation must preserve existing run styling and paragraph structure when inserting annotations.

2. **Use an embedded Hunspell-compatible engine with pinned dictionary data.** The implementation uses Spellbook 0.4.2, a pure Rust, MPL-2.0 library, with English data (MIT AND BSD) and Russian data (BSD-3-Clause) from wooorm/dictionaries. Bundle only the needed `.aff` and `.dic` files, preserve upstream notices, and pin their hashes. This avoids relying on system-installed dictionaries or external services. Spellbook describes itself as alpha and notes uneven support for advanced dictionary features. Dictionary suggestions are candidate generation only: suggestion order is not a probability estimate. The only enabled rules are four exact Russian forms reviewed from the supplied PDFs and approved during review; `холлинг` and `своеременно` additionally must appear in local dictionary suggestions. Their observed seed-set precision is 4/4, but this small set is not representative. All other English and Russian candidates remain unchanged until independently reviewed examples support class-specific precision of at least 85%.

3. **Treat extraction-specific word reconstruction separately from ordinary misspellings.** Existing deterministic hyphen and spacing cleanup remains the first step. The reviewed `посвоему` → `по-своему` exact rule handles lexical hyphenation, including when a discretionary line ending splits the form across styled runs; no generalized hyphen repair is enabled. Other correction classes require reviewed examples and class-specific precision measurement.

4. **Render the source annotation as escaped text beside the replacement.** Use `[corrected from: original]` for English and `[исправлено: original]` for Russian. The corrected word and annotation remain part of the same text flow and inherit appropriate emphasis; HTML escaping must keep the annotation inside the existing safe markup policy. Because RSS and exports derive from cached HTML, one annotated canonical representation reaches both.

5. **Keep the source alongside the correction inline and version the processing pipeline.** No separate correction-history store is needed for the requested traceability: the annotation carries the original form. Increment the processing revision when correction behavior or shipped correction data changes so unchanged source PDFs are reprocessed and cache results remain reproducible.

## Risks / Trade-offs

- [Dictionary frequency and suggestion ranking can be mistaken for calibrated confidence] → Gate each correction class on corpus-measured precision and leave unvalidated classes disabled.
- [Legitimate names, jargon, or unusual prose may be flagged] → Use conservative language gating and candidate rules; preserve text whenever eligibility is ambiguous.
- [Inline annotations add noise and can affect punctuation or screen-reader flow] → Keep annotations adjacent and consistently formatted, and validate rendered RSS and Markdown output.
- [Dictionary licenses or package size may affect distribution] → Select and review dictionary sources and licenses before bundling them; keep resources local and versioned.

## Migration Plan

Increment the PDF processing revision so cached documents regenerate on refresh. Rollback consists of disabling the correction stage and incrementing the revision again; previously published corrections remain represented in old cache entries until refresh, while a new revision regenerates them from source PDFs.
