# feed-management Specification

## Purpose

Allows one operator to create and maintain author feeds and their year/month cloud-folder sources, while preserving feed identity and control over which discovered documents are published.

## Requirements

### Requirement: Operator manages author feeds
The system SHALL provide a protected operator interface to create, view, update, archive, restore, and permanently delete author feeds.

#### Scenario: Create an author feed
- **WHEN** the operator creates a feed with its author-facing metadata
- **THEN** the system stores the feed and assigns it a persistent unique ID

#### Scenario: Update feed metadata
- **WHEN** the operator changes a feed's display name or description
- **THEN** the system preserves the feed ID and its existing RSS identity

#### Scenario: Archive and restore a feed
- **WHEN** the operator archives a feed and later restores it
- **THEN** the feed and its hierarchy are temporarily omitted from active management, refresh, and RSS publication, then resume with the same feed ID and RSS URL when restored

#### Scenario: Permanently delete a feed
- **WHEN** the operator confirms permanent deletion of a feed
- **THEN** the system permanently removes the feed, its years, months, source documents, cached content, and refresh jobs, and its RSS URL no longer publishes a channel

### Requirement: Feed months are operator-defined
The system SHALL let the operator organize each feed into years and create any subset of the twelve months for each year.

#### Scenario: Skip a month
- **WHEN** the operator creates a year without creating one of its months
- **THEN** the system treats that month as absent and does not require a cloud link for it

#### Scenario: Link a month to a cloud folder
- **WHEN** the operator assigns a supported public cloud-folder link to a month
- **THEN** the system stores the link as that month's source

#### Scenario: Archive and restore a year or month
- **WHEN** the operator archives a year or month and later restores it
- **THEN** the archived scope and its descendants are temporarily omitted from active management, scheduled refresh, and RSS publication, and become active again on restore with their stored data intact

#### Scenario: Permanently delete a year or month
- **WHEN** the operator confirms permanent deletion of a year or month
- **THEN** the system permanently removes that scope and its descendants, including month links, source documents, cached content, and refresh jobs

### Requirement: Align operator management controls
The system SHALL use consistent field widths, spacing, and action alignment across create/edit feed, add year, and link month forms. The month-linking form SHALL be visually distinct from year actions and existing month details. Each linked month SHALL present its name, source link, refresh status, and actions in clearly separated, readable groups. Layouts SHALL wrap or stack without overlap, clipped status text, or horizontal overflow on narrow screens.

#### Scenario: Review management forms
- **WHEN** the operator views feed, year, and month forms on a desktop or narrow screen
- **THEN** labels, fields, and action buttons follow consistent alignment and spacing, and action buttons do not stretch unpredictably to fill a row

#### Scenario: Review a linked month's details and controls
- **WHEN** the operator views a linked month
- **THEN** the month name and source link are easy to identify, refresh status is readable as a distinct metadata group, and month actions appear together in a separate, consistent action group with permanent deletion visually distinguished

#### Scenario: Use month controls on a narrow screen
- **WHEN** the operator views or manages a linked month on a narrow screen
- **THEN** the source, refresh metadata, and controls wrap or stack without overlapping, hiding status text, or crowding the month-linking form or year actions

### Requirement: Operator controls promotional classification
The system SHALL allow the operator to review discovered PDFs and override whether a PDF is promotional and eligible for publication.

#### Scenario: Exclude a promotional PDF
- **WHEN** a PDF is classified as promotional or the operator marks it promotional
- **THEN** the PDF is excluded from the public RSS feed

#### Scenario: Include a misclassified article
- **WHEN** the operator marks a promotional-classified PDF as publishable
- **THEN** the PDF becomes eligible for normal ingestion and publication

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

### Requirement: Provide direct URLs for dashboard views
The dashboard SHALL represent the Overview, feed list, selected feed, PDF review selection, and selected PDF review month in browser URLs that can be opened directly and restored after reload.

#### Scenario: Open a dashboard section directly
- **WHEN** an authenticated operator opens a URL for Overview, the feed list, a selected feed, or PDF review
- **THEN** the dashboard displays the view represented by that URL

#### Scenario: Restore a selected PDF review month
- **WHEN** an authenticated operator opens or reloads a PDF review URL containing a feed, year, and month
- **THEN** PDF review restores that selection and its documents

#### Scenario: Preserve partial PDF review selections
- **WHEN** an authenticated operator opens or reloads a PDF review URL containing a feed, or a feed and year
- **THEN** PDF review restores the selected hierarchy level and lets the operator continue choosing a year or month

#### Scenario: Navigate with browser history
- **WHEN** an operator changes dashboard views or PDF review selection and uses browser Back or Forward
- **THEN** the dashboard restores the view and selection represented by the active URL

#### Scenario: Open a URL for a missing selection
- **WHEN** a URL refers to a feed, year, or month that is unavailable
- **THEN** the dashboard shows a usable feed or PDF review view and communicates that the requested selection is unavailable

### Requirement: Remember the desktop sidebar state
The dashboard SHALL let operators collapse or expand the desktop sidebar and persist that preference across browser reloads.

#### Scenario: Collapse and reload the sidebar
- **WHEN** an operator collapses the desktop sidebar and reloads the dashboard
- **THEN** the sidebar remains collapsed

#### Scenario: Expand and reload the sidebar
- **WHEN** an operator expands the desktop sidebar and reloads the dashboard
- **THEN** the sidebar remains expanded

### Requirement: Keep dashboard navigation focused
The dashboard SHALL provide a link from the Kpapers brand to Overview and omit the refresh scheduler banner and footer promotional labels.

#### Scenario: Return to Overview from the brand
- **WHEN** an operator activates the Kpapers brand in the dashboard sidebar
- **THEN** the dashboard navigates to Overview

#### Scenario: View dashboard chrome
- **WHEN** an operator views the dashboard sidebar and footer
- **THEN** the refresh scheduler banner, “Kpapers operator desk” label, and “Feeds update on your schedule” message are absent

### Requirement: Show current source folder and review counts on Overview
The Overview SHALL show the number of configured source folders and PDFs needing operator attention, and SHALL distinguish a zero count from unavailable count data.

#### Scenario: Count configured source folders
- **WHEN** Overview loads successfully
- **THEN** Source folders shows the number of configured month folders under active feeds and years

#### Scenario: Count PDFs needing review
- **WHEN** Overview loads successfully
- **THEN** Needs review shows the number of PDFs with an unknown classification or pending or failed extraction and classification processing

#### Scenario: Show zero counts
- **WHEN** the operator has no configured source folders or no PDFs needing review
- **THEN** the corresponding Overview count displays zero

#### Scenario: Overview counts cannot be loaded
- **WHEN** the dashboard cannot retrieve Overview counts
- **THEN** it communicates that the counts are unavailable instead of displaying a placeholder that looks like a count
