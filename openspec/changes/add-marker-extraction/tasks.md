# Tasks

## 1. Configuration

- [x] 1.1 Add and validate the global engine default and nullable feed override, including migration and backward-compatible API updates; verify defaults, inheritance, overrides, invalid values and existing-client updates with backend tests.
- [x] 1.2 Add the inherit/Pdfium/Marker selector to feed creation and editing, and verify persistence and validation with UI tests.

## 2. Marker worker and extraction

- [x] 2.1 Pin Marker and its runtime, build a private long-lived worker with bounded inputs, outputs, concurrency and execution time, and verify startup, conversion and error handling with worker tests.
- [x] 2.2 Evaluate Marker's page-aware output on representative native-text, scanned, advertisement and Sponsr PDFs; establish and test a structured response contract sufficient for page-level cleanup and classification.
- [x] 2.3 Integrate engine selection into the shared manual and scheduled extraction path; verify Marker handles native-text PDFs and worker errors do not invoke Pdfium.
- [x] 2.4 Adapt cleanup, title recovery, classification and safe HTML generation for Marker output; verify existing publication requirements and unsafe markup handling against representative fixtures for both engines.
- [x] 2.5 Include engine and processing revision in cache identity and preserve last successful cached content on failure; test unchanged sources, changes to global and feed selection, forced refresh and failed reprocessing.

## 3. Deployment and integration

- [x] 3.1 Add optional private worker deployment and configuration, including model provisioning, resource needs, pinned licenses and Pdfium-only operation; verify documented startup and rollback procedures.
- [x] 3.2 Run backend and UI suites and an end-to-end refresh using each engine; verify RSS output, cleanup, cache reuse, engine switching and worker-unavailable behavior.

## 4. Verification remediation

- [x] 4.1 Keep the last successful entry available in RSS and its public document route after extraction failure, while reporting the failure separately; verify failure and subsequent matching-cache recovery with integration tests.
- [x] 4.2 Prevent an OCR-disabled Marker retry from replacing cached content with an incomplete mixed native-text and scanned document; verify incomplete or unverifiable retries fail and preserve the prior cache.
- [x] 4.3 Preserve indented notice structure, paragraph boundaries and mixed-emphasis styles in Marker output; diagnose the failing Marker E2E document and conversion error, cover a mixed-emphasis notice, and rerun the Sponsr E2E successfully with following body text outside the blockquote.
- [x] 4.4 Ensure timed-out Marker conversion work stops or its worker restarts and releases capacity even when the client disconnects or the timeout response cannot be written; verify the configured supervisor restarts the worker and a subsequent conversion succeeds.
- [x] 4.5 Select OCR unless native text-layer completeness is established, including small scanned regions, images nested beyond PDF object traversal limits, visible lettering drawn as vector paths, and explicitly configured OCR-disabled mode. Ensure Marker's automatic text-layer decision cannot skip required OCR; test nested images and vector lettering with real PDFs, and add a persistent automated test using the pinned Marker runtime that converts a same-page native header and scanned text fixture, including a case where automatic text-layer selection could skip OCR. Verify safe failure retains the prior cache.
