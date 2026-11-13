# Design

## Context

The Rust API currently calls Pdfium directly during extraction, generates HTML and first-page text, and caches the result using a pipeline revision and source fingerprint. Existing cleanup depends on page boundaries and PDF layout, and classification uses first-page text. Scheduled and manual extraction share the same path.

## Goals / Non-Goals

**Goals:** Preserve the existing publication contract while allowing a feed to choose a different extraction engine. Keep existing installations operational without Marker.

**Non-Goals:** Automatically fall back between engines, perform bulk immediate reprocessing on configuration changes, or promise identical formatting between engines.

## Decisions

1. Store a nullable, validated per-feed engine override; null means inherit. Use a validated deployment environment setting for the global default, defaulting to Pdfium. Expose the override in feed API and management forms, preserving existing clients that omit it. This follows the existing global scheduling defaults and per-feed override pattern. A global settings UI would add persistence and management scope without being necessary here.
2. Run Marker in a separate long-lived Python worker on a private deployment network, loading models once. The API sends bounded PDF data and receives structured page-aware output with page count and text/formatting needed for cleanup. Limit concurrent work, request size, response size and execution time; treat worker failures as extraction errors. This isolates model resources and avoids reloading them for every document. A subprocess per PDF is simpler but expensive; embedding Python in Rust couples runtimes.
3. Adapt both engine outputs into a shared processing contract or implement equivalent cleanup over Marker's structured pages. Preserve boundary-page filtering, page labels, Sponsr cleanup and title recovery, classification and semantic safe HTML. Validate page boundaries and styling against real fixtures before committing to a particular Marker output format. Sanitize generated HTML explicitly; never trust worker markup. Existing Pdfium-specific layout heuristics may need equivalent Marker-aware handling rather than a direct reuse.
4. Include effective engine and its processing revision in cache fingerprints. At each normal or forced extraction, resolve the current selection before checking the cache. On a change, the next refresh reprocesses the document; a failed extraction leaves the prior cache intact and reports failure. No implicit Pdfium fallback hides Marker errors.
5. Pin the Python package and model/runtime versions, document installation, model acquisition and licenses, and support a Pdfium-only deployment. The worker must not be exposed publicly; limit its access and avoid logging document contents.

## Risks / Trade-offs

- [Marker structured output may not preserve cues required by current cleanup] -> Verify representative multi-page, advertisement, Sponsr, formatting and scanned fixtures; adapt processing and retain acceptance tests across engines.
- [Model startup, CPU, memory and disk demands] -> Load once, bound concurrency and timeouts, document hardware and model provisioning, and leave Pdfium as the default.
- [Engine changes can trigger significant refresh work] -> Reprocess lazily on refresh and retain cached successful content on errors.
- [Worker failure or malformed output] -> Validate responses, sanitize HTML, bound resource usage, and surface errors without fallback.
- [Licensing and model availability] -> Document exact pinned package and model terms and provision models for the target environment before deployment.

## Migration Plan

Add the nullable feed field with existing rows inheriting Pdfium by default. Deploy the optional worker and configure its internal endpoint before selecting Marker. Switching the default or an override causes lazy cache replacement. Roll back by selecting Pdfium; subsequent refreshes regenerate content, while existing successful cached content remains available during failures. Database rollback must account for the new nullable field and API compatibility.
