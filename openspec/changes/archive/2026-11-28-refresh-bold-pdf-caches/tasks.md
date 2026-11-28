# Tasks

## 1. Invalidate extraction caches after parser changes

- [x] 1.1 Advance the PDF extraction pipeline revision and update its revision assertion; verify an unchanged source with an older cache key is reprocessed by the existing cache regression test.

## 2. Verify forced parsing reaches published outputs

- [x] 2.1 Extend the extraction API regression setup with the March 10 reference PDF and stale cached HTML; invoke forced parsing and verify the stored sanitized HTML contains the selected `<strong>` passage while ordinary text remains unwrapped.
- [x] 2.2 From the refreshed cached record, verify the RSS item contains the selected `<strong>` passage and the feed Markdown ZIP contains its `**` equivalent; verify both outputs leave ordinary text unwrapped.
- [x] 2.3 Normalize article headings before caching; verify the March 10 RSS item and Markdown export share exactly one H1 and valid H2/H3 hierarchy, and verify a document without a content H1 falls back to its filename title in both outputs.
