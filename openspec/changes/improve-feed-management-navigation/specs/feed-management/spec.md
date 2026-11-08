# Spec Delta

## MODIFIED Requirements

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

## ADDED Requirements

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
