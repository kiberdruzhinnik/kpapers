# Proposal

## Why

Spec validation reports a placeholder deployment purpose and an overlong PDF heading requirement. These warnings obscure useful validation feedback and make the affected specifications harder to read.

## What Changes

- Replace the placeholder purpose in the container deployment main spec with a description grounded in its existing API routing requirement.
- Split the PDF heading requirement into focused, shorter requirements while preserving all existing normative behavior and scenarios.
- Verify the main specs with strict OpenSpec validation and compare the rewritten requirements with the original contract.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. This is an editorial restructuring of existing main specs; there is no intended behavior change or delta spec.

## Impact

- `openspec/specs/container-deployment/spec.md` and `openspec/specs/pdf-ingestion/spec.md` are the only planned implementation edits.
- Application code, deployment behavior, and existing scenario meaning remain unchanged.
