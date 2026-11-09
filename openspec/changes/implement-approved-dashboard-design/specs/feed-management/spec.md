# Spec Delta

## ADDED Requirements

### Requirement: Present the approved operator workspace
The dashboard SHALL follow the approved layout, typography, colors, spacing, and responsive behavior in `docs/design/DESIGN.md`, with a persistent desktop sidebar and accessible mobile navigation.

#### Scenario: Use the workspace across screen sizes
- **WHEN** the operator visits any management screen at 320, 375, 768, 1024, or 1440px width or in landscape
- **THEN** controls remain accessible, content wraps without horizontal overflow, and navigation retains its active route and saved collapse preference

### Requirement: Present actionable publishing overview
Overview SHALL show three actual metrics, publishing shortcuts, and a review queue that distinguishes loading, unavailable, clear, and attention states. First-feed onboarding SHALL appear only when no active feeds exist.

#### Scenario: Existing feeds with a clear queue
- **WHEN** active feeds exist and the review count is confirmed zero
- **THEN** Overview shows publishing shortcuts and “No documents need attention” without first-feed setup instructions

#### Scenario: Review data unavailable
- **WHEN** the summary request fails
- **THEN** the queue reports unavailable data and does not imply that all documents are clear

### Requirement: Separate feed list and detail screens
The feed list SHALL contain restrained feed rows grouped by Active and Archived tabs. Opening a row SHALL navigate to a separate detail screen with All feeds navigation, stable RSS address, metadata editing, and scoped management actions.

#### Scenario: Open and edit a feed
- **WHEN** the operator opens a feed from the list and saves edited metadata in a focused dialog
- **THEN** the detail screen retains the feed identity and RSS address and reports the server outcome

#### Scenario: Inspect archived feeds
- **WHEN** the operator opens Archived
- **THEN** archived rows offer restoration and separately confirmed permanent deletion without exposing hierarchy forms on the list

### Requirement: Focus source management controls
Feed detail SHALL show one Source folders section with Add year and independently collapsible years. Connect month SHALL open a focused form. Manage source SHALL disclose the source URL, local refresh timestamps, latest result, and existing scoped actions.

#### Scenario: Manage a configured month
- **WHEN** the operator expands a year and opens Manage source for a month
- **THEN** its metadata and refresh, archive, restore, and separately confirmed deletion actions are available without changing their scope

#### Scenario: Preserve hierarchy preferences
- **WHEN** the operator returns to a feed or reloads it
- **THEN** only years explicitly expanded in the existing browser preference are open

### Requirement: Explain feed creation validation
The dedicated creation screen SHALL use the approved narrow form, preserve existing fields and values on failure, expose linked field errors, and focus an error summary on invalid submission. Pending submissions SHALL be disabled; Cancel SHALL return to the list and success SHALL open the created feed.

#### Scenario: Submit an invalid feed
- **WHEN** the operator submits missing or invalid values
- **THEN** the focused summary links to invalid fields and each control exposes its associated error

### Requirement: Distinguish PDF classification from publication decisions
PDF review SHALL retain labelled dependent selectors, deep links, document-specific sources, extraction state, and independent classification and publication decision feedback. Failed decisions SHALL retain the prior value; inclusion SHALL not bypass successful extraction.

#### Scenario: Override a promotional document
- **WHEN** the operator selects Include in feed
- **THEN** its automatic classification remains visible separately from the saved override and extraction eligibility

#### Scenario: Decision save fails
- **WHEN** a document decision cannot be saved
- **THEN** that row reports failure and retains the prior saved decision
