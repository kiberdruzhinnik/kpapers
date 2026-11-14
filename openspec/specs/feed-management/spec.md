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
The dashboard SHALL represent Overview, feed list, selected feed, feed creation, PDF review selection, and selected PDF review month in directly openable browser URLs. Overview SHALL use `/manage/`. Feed list, selected feed, and creation SHALL use `/manage/feeds`, `/manage/feeds/<id>`, and `/manage/feeds/new`. PDF review SHALL use `/manage/pdf-review` with optional `/<feed-id>[/<year>[/<month-id>]]` selections. Public `/feeds/<id>` SHALL retain RSS behavior. Reloads SHALL restore the addressed view, without requiring restoration of unsaved form values.

#### Scenario: Open a dashboard section directly
- **WHEN** an authenticated operator opens a URL for Overview, the feed list, a selected feed, feed creation, or PDF review
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

#### Scenario: Open Overview and PDF review under the management prefix
- **WHEN** an authenticated operator opens or reloads `/manage/` or `/manage/pdf-review` through the web server
- **THEN** the dashboard displays Overview or PDF review respectively

#### Scenario: Restore PDF review selection under the management prefix
- **WHEN** an authenticated operator opens or reloads `/manage/pdf-review/<feed-id>/<year>/<month-id>`
- **THEN** PDF review restores the selected feed, year, month, and its documents

#### Scenario: Navigate dashboard links within the management prefix
- **WHEN** the operator uses the brand link, section navigation, or PDF review selection controls
- **THEN** the resulting dashboard URL uses `/manage/` and preserves the relevant selection

#### Scenario: Reload feed management through the deployed web server
- **WHEN** an authenticated operator opens or reloads `/manage/feeds/<id>` for an available feed through the web server
- **THEN** the dashboard displays the selected feed instead of RSS XML

#### Scenario: Fetch an existing RSS subscription URL
- **WHEN** a reader fetches `/feeds/<id>` for an active published feed
- **THEN** the server returns that feed's RSS channel at the same URL

#### Scenario: Return from PDF review to feed management
- **WHEN** the operator activates Back to feeds from a selected feed's PDF review
- **THEN** the dashboard navigates to `/manage/feeds/<id>` for that feed

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

### Requirement: Create feeds on a dedicated page
The dashboard SHALL provide a protected feed creation page at `/manage/feeds/new`, reachable from Create feed controls on Overview and Feeds. It SHALL offer the existing author name, optional description, and optional refresh interval fields and validation. Feed management SHALL offer a creation link instead of an embedded creation form.

#### Scenario: Start creation from Overview or Feeds
- **WHEN** an authenticated operator activates Create feed on Overview or Feeds
- **THEN** the dedicated creation page opens with the creation form

#### Scenario: Create and configure a feed
- **WHEN** the operator successfully submits valid feed details
- **THEN** the system creates the feed and navigates to `/manage/feeds/<id>` for the created feed, ready for year and month configuration
- **AND** the feed appears in the dashboard's feed list

#### Scenario: Cancel feed creation
- **WHEN** the operator activates Cancel on the creation page
- **THEN** the dashboard returns to `/manage/feeds` without creating a feed

#### Scenario: Correct a failed creation submission
- **WHEN** validation fails or the creation request fails
- **THEN** the creation page retains entered values and displays an error so the operator can correct or retry the submission

#### Scenario: Open creation before sign-in
- **WHEN** an unauthenticated visitor opens `/manage/feeds/new`
- **THEN** the dashboard shows sign-in without exposing the creation form
- **AND** successful sign-in opens the creation page

### Requirement: Remember independently collapsible feed years
The dashboard SHALL let the operator independently expand and collapse each feed year. A year with no saved preference SHALL start collapsed, including newly created years. The browser SHALL remember each feed/year preference across reloads and navigation. Collapsing SHALL hide month details and the month-linking form while leaving the year heading, status, and year actions visible.

#### Scenario: View years without saved preferences
- **WHEN** the operator opens a feed whose years have no saved expansion preferences
- **THEN** every year is collapsed, including archived years

#### Scenario: Remember mixed year states
- **WHEN** the operator expands one year, leaves another collapsed, then reloads or leaves and returns to the feed
- **THEN** each year restores its own expanded or collapsed state

#### Scenario: Keep preferences separate across feeds
- **WHEN** the operator expands a year in one feed and opens the same calendar year in another feed without a saved preference
- **THEN** the second feed's year remains collapsed

#### Scenario: Toggle a year using the keyboard
- **WHEN** the operator focuses a year's expansion control and activates it with the keyboard
- **THEN** the year toggles and its control exposes the current expanded state to assistive technology

#### Scenario: Browser persistence is unavailable
- **WHEN** browser storage is blocked or a saved preference is invalid
- **THEN** the year view remains usable and defaults to collapsed for an unreadable preference
- **AND** the operator can still expand and collapse years during that view

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
