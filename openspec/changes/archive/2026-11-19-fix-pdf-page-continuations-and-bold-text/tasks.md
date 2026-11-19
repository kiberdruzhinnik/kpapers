# Tasks

## 1. Fix page-boundary paragraph continuation

- [x] 1.1 Reproduce the supplied page transition using extracted line text, style, and geometry; identify which continuation or block-splitting condition rejects it, and add a regression test that currently captures the unwanted paragraph split.
- [x] 1.2 Correct the narrow page-boundary decision so `потому что` followed by `любой вопрос...` is one paragraph, and verify with regression tests that genuine page-boundary paragraph breaks remain distinct.

## 2. Preserve bold styling

- [x] 2.1 Trace bold metadata from extracted PDF words through styled-line conversion and HTML rendering; add focused regression coverage for a bold word between regular words and verify `<strong>` wraps only the bold word.
- [x] 2.2 Correct the metadata or rendering stage identified by the regression, and verify bold, italic, nested styling, and adjacent same-style run tests still pass.

## 3. End-to-end PDF regression

- [x] 3.1 Process the supplied PDF locally and verify the sentence is continuous and the emphasized `любой` is rendered as italic, without adding the full article PDF to repository fixtures; verify bold parsing separately with a mixed-style regression test.
- [x] 3.2 Run the relevant PDF processing test suite and verify existing paragraph, heading, emphasis, and safe-HTML behavior remains intact.
