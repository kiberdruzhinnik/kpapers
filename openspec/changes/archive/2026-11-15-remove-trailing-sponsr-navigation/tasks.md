# Tasks

## 1. Trailing disclaimer cleanup

- [x] 1.1 Add page-aware trailing disclaimer recognition and trim subsequent navigation and related-content lines while preserving the full disclaimer block; verify unit coverage for both supplied page patterns, a teaser crossing a page boundary, preserved article/disclaimer text, and no-marker preservation.
- [x] 1.2 Advance the PDF processing revision and verify the cache fingerprint changes so cached entries are reprocessed.
- [x] 1.3 Verify the processed text and HTML both omit the trailing navigation and teaser while retaining the disclaimer, using focused PDF processing tests.

## 2. Hyphenated words across line breaks

- [x] 2.1 Preserve compounds when adjacent extracted lines split after a hyphen; add regression coverage for `по-товарищески` and `чем-нибудь` in extracted text and HTML, plus an ordinary line-break control, and verify the focused PDF processing tests pass.
- [x] 2.2 Advance the PDF processing revision from v18 to v19 and verify the cache fingerprint no longer matches v18 entries.
- [x] 2.3 Reproduce the reported `объяснения-` / `мастерключа её следующая формула:` wrap through the relevant extraction path using a minimized PDF fixture or captured extracted-line structure; ensure extracted text and HTML join it correctly, fix the behavior if needed, retain an ordinary line-break control, and verify focused PDF processing tests pass.
- [x] 2.4 Confirm whether task 2.3 changes extraction behavior; it does not, so retain v19 and verify the cache fingerprint remains distinct from v18 entries.
