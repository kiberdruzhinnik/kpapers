# Tasks

## 1. Configuration

- [ ] 1.1 Add and validate the global engine default and nullable feed override, including migration and backward-compatible API updates; verify defaults, inheritance, overrides, invalid values and existing-client updates with backend tests.
- [ ] 1.2 Add the inherit/Pdfium/Marker selector to feed creation and editing, and verify persistence and validation with UI tests.

## 2. Marker worker and extraction

- [ ] 2.1 Pin Marker and its runtime, build a private long-lived worker with bounded inputs, outputs, concurrency and execution time, and verify startup, conversion and error handling with worker tests.
- [ ] 2.2 Evaluate Marker's page-aware output on representative native-text, scanned, advertisement and Sponsr PDFs; establish and test a structured response contract sufficient for page-level cleanup and classification.
- [ ] 2.3 Integrate engine selection into the shared manual and scheduled extraction path; verify Marker handles native-text PDFs and worker errors do not invoke Pdfium.
- [ ] 2.4 Adapt cleanup, title recovery, classification and safe HTML generation for Marker output; verify existing publication requirements and unsafe markup handling against representative fixtures for both engines.
- [ ] 2.5 Include engine and processing revision in cache identity and preserve last successful cached content on failure; test unchanged sources, changes to global and feed selection, forced refresh and failed reprocessing.

## 3. Deployment and integration

- [ ] 3.1 Add optional private worker deployment and configuration, including model provisioning, resource needs, pinned licenses and Pdfium-only operation; verify documented startup and rollback procedures.
- [ ] 3.2 Run backend and UI suites and an end-to-end refresh using each engine; verify RSS output, cleanup, cache reuse, engine switching and worker-unavailable behavior.
