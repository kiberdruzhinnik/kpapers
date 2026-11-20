# Proposal

## Why

The supplied CrimsonAnalytics PDF is parsed with an incomplete headline and a disclaimer that does not match the requested HTML. Readers need the full wrapped headline as one title, followed by the disclaimer as a separate bold paragraph while keeping the literal Markdown asterisks from the source.

## What Changes

- Ensure this PDF's wrapped headline is reconstructed as one `<h1>`.
- Preserve `**это не инвестиционные рекомендации**` as the literal text inside a bold paragraph, producing `<p><strong>**это не инвестиционные рекомендации**</strong></p>`.
- Update the PDF ingestion requirement that currently says matched Markdown markers are removed, which conflicts with this requested output.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pdf-ingestion`: Specify complete wrapped-title output and preservation of literal Markdown markers in the bold disclaimer paragraph.

## Impact

The PDF text-to-HTML publication pipeline and its `pdf-ingestion` specification are affected. The expected output remains within the existing safe HTML structure and uses the project's existing `<strong>` convention for bold text.
