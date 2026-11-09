# Tasks

## 1. Remove matching page labels

- [x] 1.1 Filter standalone numeric page lines against each page's position regardless of extracted line order; add unit cases for leading and later labels, nonmatching numbers, and numbers embedded in content, and verify the focused PDF-processing tests pass.
- [x] 1.2 Match page labels against the page's one-based position after leading boundary advertisement pages are removed so printed numbering that starts at `1` on the first content page is omitted; add a regression case and verify the focused PDF-processing tests pass.

## 2. Reduce false heading classification

- [x] 2.1 Replace the minimum-font-size baseline and size-only candidate rule with a representative body-size baseline and corroborating typography/layout cues; add regression cases for ordinary prominent prose, title, section, subsection, wrapped headings, and inline emphasis, and verify the focused PDF-processing tests pass.
- [x] 2.2 Review generated HTML for the supplied Russian PDF and confirm the title remains `<h1>`, the shown prose paragraphs remain `<p>`, inline italics are preserved, and no standalone matching page labels remain.

## 3. Refresh cached extraction

- [x] 3.1 Increment the extraction pipeline revision and verify the cache fingerprint changes for existing source fingerprints while unchanged revisions remain stable.
