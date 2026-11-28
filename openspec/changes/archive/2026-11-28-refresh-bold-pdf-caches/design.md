# Design

## Context

See proposal.md for the observed output. The extraction cache key already combines the source fingerprint with `EXTRACTION_PIPELINE_REVISION`; it currently identifies this parser as revision 37. Feed Markdown export prepends a filename-derived H1 and then converts cached sanitized HTML, which may itself contain an H1. RSS publishes cached HTML directly. The document force-parse UI requests `force=true`; successful extraction replaces the cached text and HTML.

## Goals / Non-Goals

**Goals:**
- Cause unchanged PDFs with cached results from before synthetic-bold recognition to be processed by the updated pipeline.
- Prove a forced parse updates cached HTML and that the updated emphasis appears in RSS and Markdown ZIP exports.
- Ensure cached article HTML, RSS content, and Markdown ZIP files share one H1 and a valid H1/H2/H3 hierarchy.

**Non-Goals:**
- Change the bold detection rule or the HTML/Markdown emphasis syntax.
- Change RSS item identity, export filtering, or the database schema.

## Decisions

- **Advance the extraction pipeline revision when shipping the formatting correction.** The existing cache key already includes this revision, so changing it makes normal refreshes reprocess unchanged source files. This uses the established invalidation mechanism instead of clearing cached rows or adding a migration.
- **Keep forced parsing as an explicit cache bypass and verify the persisted result.** The force path should run the current parser and replace cached extracted text and HTML only after successful processing. A failed forced parse must continue retaining the previous successful cache.
- **Exercise the stored-content publication path with the reference PDF.** Seed stale cached content for the reference document, run the forced extraction path, then assert the refreshed `<strong>` content in RSS and `**` in the feed Markdown ZIP. Also assert ordinary text remains unwrapped. This distinguishes parser correctness from cache and exporter behavior.
- **Normalize headings once in cached article HTML for both publishers.** Use the first content H1 as the sole H1 when present, otherwise use the filename-derived title. Normalize later headings to H2/H3 without level skips. RSS can publish the canonical HTML directly, and Markdown conversion can preserve its hierarchy without prepending a second H1. An alternative of normalizing separately in each publisher risks heading differences between RSS and downloads.

Alternatives considered: clearing all cached content at deployment would also refresh formatting but risks losing last-good data and does not fix future invalidation omissions. Relying only on a manual force action leaves other unchanged PDFs stale until individually processed.

## Risks / Trade-offs

- [A revision bump can cause many unchanged PDFs to download and reprocess after deployment] → Use the existing refresh queue and retain the last successful cache if processing fails.
- [A force-path regression could still leave the cache stale despite the revision bump] → Cover the force request through the cached database and publication outputs in regression tests.

## Migration Plan

Ship the parser behavior and a new extraction pipeline revision together. Existing entries are reprocessed as refreshes encounter their stale cache revision; failed processing keeps their previous successful output. Operators can still force individual entries to process immediately. Rolling back the application restores the prior pipeline revision and safely reuses the prior compatible cache key.
