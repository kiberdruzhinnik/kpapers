# Spec Delta

## ADDED Requirements

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
