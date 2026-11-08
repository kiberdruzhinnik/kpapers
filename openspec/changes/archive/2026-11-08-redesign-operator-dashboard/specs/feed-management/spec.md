# Spec Delta

## ADDED Requirements

### Requirement: Provide a cohesive operator dashboard
The system SHALL present the protected operator workflows in a clean, responsive dashboard with consistent navigation, controls, and feedback. The dashboard SHALL make sign-in, feed and hierarchy management, refresh state, PDF review, and archive, restore, and permanent deletion available without changing their existing outcomes.

#### Scenario: Sign in to the operator dashboard
- **WHEN** an operator submits valid credentials
- **THEN** the protected dashboard appears with the operator's active feeds and access to feed creation

#### Scenario: Keep management unavailable before sign-in
- **WHEN** an unauthenticated visitor opens the operator interface
- **THEN** the sign-in view is shown and protected management content is not exposed

#### Scenario: Manage a selected feed and its hierarchy
- **WHEN** the operator selects a feed
- **THEN** the dashboard presents its metadata, stable RSS URL, years, configured months, refresh state, and available actions in clearly grouped sections

#### Scenario: Copy a feed's RSS address
- **WHEN** the operator activates the copy control beside a feed's RSS address
- **THEN** the displayed stable RSS address is copied to the clipboard and the dashboard reports success
- **AND** if the browser cannot copy through its clipboard API, the dashboard uses a compatible fallback or clearly reports that copying failed

#### Scenario: Review discovered PDFs
- **WHEN** the operator opens PDF review for a configured month
- **THEN** each document's classification, extraction state, and publication choice are readable, and changing the choice reports whether it was saved

#### Scenario: Open an individual source PDF
- **WHEN** the operator activates "View source PDF" for a discovered document
- **THEN** the selected PDF opens from a document-specific URL rather than the containing provider folder or a generic provider page
- **AND** the link works for documents visible in review even when they are not currently published in the feed

#### Scenario: Navigate PDF review by feed, year, and month
- **WHEN** the operator browses PDF review
- **THEN** available documents are organized through a feed, then year, then month hierarchy
- **AND** selecting a month shows that month's documents and preserves the selected feed, year, and month context

#### Scenario: Use the dashboard on a narrow screen
- **WHEN** the operator uses the dashboard at a narrow viewport
- **THEN** navigation, forms, status details, and actions reflow without clipped controls, overlap, or horizontal page overflow

#### Scenario: Receive feedback for a failed management action
- **WHEN** a management request fails
- **THEN** the dashboard communicates the failure and keeps the operator on a usable view from which the action can be retried
