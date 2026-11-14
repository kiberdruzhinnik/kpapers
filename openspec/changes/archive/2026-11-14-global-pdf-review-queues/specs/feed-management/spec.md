# Spec Delta

## MODIFIED Requirements

### Requirement: Present actionable publishing overview
Overview SHALL show three actual metrics, publishing shortcuts, and a review queue that distinguishes loading, unavailable, clear, and attention states. First-feed onboarding SHALL appear only when no active feeds exist.

#### Scenario: Existing feeds with a clear queue
- **WHEN** active feeds exist and the active review count is confirmed zero
- **THEN** Overview shows publishing shortcuts and “No documents need attention” without first-feed setup instructions

#### Scenario: Active documents need review
- **WHEN** the active review count is greater than zero
- **THEN** Overview shows the number of active documents needing attention and provides a link that opens the active Needs review queue directly

#### Scenario: Review data unavailable
- **WHEN** the summary request fails
- **THEN** the queue reports unavailable data and does not imply that all documents are clear

## ADDED Requirements

### Requirement: Provide global active and archived PDF review queues
PDF review SHALL provide global queues of documents needing operator attention, separated according to the archive state of each document's feed, year, and month.

#### Scenario: Review active documents needing attention
- **WHEN** the operator opens the active Needs review queue
- **THEN** it lists documents under active feeds, years, and months whose extraction is pending or failed or whose promotional classification is unknown
- **AND** each document shows enough feed, year, and month context to identify its source and retains its existing source, extraction, and publication decision controls

#### Scenario: Review archived documents needing attention
- **WHEN** the operator opens the Archived review queue
- **THEN** it lists documents needing attention whose feed, year, or month is archived
- **AND** documents in this queue are excluded from the active review count on Overview

#### Scenario: A source is restored
- **WHEN** an archived feed, year, or month containing a document needing attention is restored so that all its scopes are active
- **THEN** the document appears in the active queue and is removed from the archived queue

#### Scenario: No documents need attention in a queue
- **WHEN** the selected queue contains no documents needing attention
- **THEN** PDF review communicates that the queue is empty
