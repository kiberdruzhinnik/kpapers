# Tasks

## 1. Preserve the requested headline and disclaimer output

- [x] 1.1 Update PDF HTML generation so the full wrapped CrimsonAnalytics headline remains one `<h1>` and the following disclaimer is emitted as its own `<p><strong>` while retaining literal `**` markers; add regression coverage that asserts the exact expected HTML.
- [x] 1.2 Verify title and disclaimer formatting remains correct when text and asterisks span adjacent PDF styled runs, and unrelated unmatched markers remain ordinary text; add focused unit coverage for both behaviors.

## 2. Refresh cached extraction output

- [x] 2.1 Increment the PDF processing revision and update its cache fingerprint assertion; verify changed revisions produce a distinct cache identity.
- [x] 2.2 Run the focused PDF processing tests and verify the supplied headline and disclaimer produce the required HTML without altering surrounding article content.
