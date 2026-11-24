# Spec Delta

## ADDED Requirements

### Requirement: Align PDF review mode controls with Feeds
PDF review SHALL present its review modes in the same compact segmented-tab style used by the Feeds Active and Archived tabs, without a page-level back button.

#### Scenario: View PDF review modes
- **WHEN** an operator opens PDF review
- **THEN** Needs review, Archived, and Browse by source appear as compact segmented tabs and no page-level back button is shown

#### Scenario: Select a PDF review mode
- **WHEN** an operator selects a PDF review mode
- **THEN** the selected mode is clearly indicated and the existing mode behavior and URL are preserved
