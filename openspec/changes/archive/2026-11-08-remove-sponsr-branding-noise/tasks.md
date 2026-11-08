# Tasks

## 1. Filter the known branding line

- [x] 1.1 Add normalized exact-line recognition for `Кримсон Аналитика – CrimsonAnalytics` across every extracted page, then verify unit tests show the line removed from both plain text and HTML while surrounding lines, embedded mentions, and unrelated branding remain.

## 2. Cover document output and cache refresh

- [x] 2.1 Add a regression fixture based on the supplied PDF (or a minimized representative fixture) and verify the branding is absent from extraction output on its observed page while adjacent article content remains.
- [x] 2.2 Advance the extraction fingerprint to `pdf-text-v10` and verify existing v9 cache entries miss once and subsequent v10 fingerprints remain stable.
- [x] 2.3 Run the focused PDF processing and cache tests, and verify the regression fixture passes using the pinned Linux Pdfium runtime.
