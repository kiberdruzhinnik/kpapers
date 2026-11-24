# Tasks

## 1. Evaluate local correction resources and confidence

- [x] 1.1 Compare local Russian and English dictionary/checker options, including licensing and packaging, and document the selected resources and their versioning approach.
- [x] 1.2 Build a reviewed seed evaluation set from the supplied PDF extraction output, measure precision for each exact reviewed Russian form, and keep every unvalidated general correction class disabled. The seed set is too small to establish representative class precision.

## 2. Implement conservative local correction

- [x] 2.1 Add local Russian and English dictionary checking; apply only the exact reviewed Russian forms and leave all unvalidated or ambiguous candidates unchanged.
- [x] 2.2 Add adjacent English and Russian source annotation formatting while preserving text style, paragraph structure, and safe HTML escaping; verify rendered corrections and untouched text with unit tests.
- [x] 2.3 Regenerate `full_text`, first-page text, HTML, and Markdown from the same corrected representation and increment cache processing identity; verify all representations stay aligned and changed revisions invalidate old cache entries.

## 3. Verify publication behavior

- [x] 3.1 Verify RSS and Markdown exports include inline source annotations for both language formats and preserve safe markup using integration tests.
- [x] 3.2 Document the local correction resources, licenses, evaluated exact forms, and measured seed precision results; verify documentation matches the shipped resources and enabled rules.
