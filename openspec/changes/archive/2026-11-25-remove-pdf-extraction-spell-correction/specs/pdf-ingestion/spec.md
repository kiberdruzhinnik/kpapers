# Spec Delta

## REMOVED Requirements

### Requirement: Correct high-confidence Russian and English extraction errors locally
**Reason**: PDF extraction must preserve extracted wording and no longer performs spelling or word-form correction.
**Migration**: None. Extracted text is published without automatic corrections.

### Requirement: Annotate applied corrections in published text
**Reason**: The PDF extraction pipeline no longer applies corrections to annotate.
**Migration**: None. Published text contains the extracted wording without correction annotations.

### Requirement: Reprocess cached content when local correction processing changes
**Reason**: There is no longer local correction processing or correction data to include in cache identity.
**Migration**: The existing general PDF processing revision requirement remains in force. Advance that revision once when removing the correction stage so cached output is regenerated on the next successful refresh.
