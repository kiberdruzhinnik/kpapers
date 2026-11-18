# Spec Delta

## MODIFIED Requirements

### Requirement: Focus source management controls
Feed detail SHALL show a single Source folders section with Add year and independently collapsible, compact year groups. Expanded years SHALL show configured months as concise rows with their state, latest result, PDF review action, and a source-details disclosure. Connect month SHALL open a focused form. Opening source details SHALL reveal the source URL, refresh timestamps, latest result or error, and existing scoped actions.

#### Scenario: Scan a feed's source hierarchy
- **WHEN** the operator views a feed with configured years and months
- **THEN** the page presents compact year groups and concise month rows whose month, state, latest result, and PDF review action are easy to scan

#### Scenario: Manage a configured month
- **WHEN** the operator expands a year and opens Source details for a month
- **THEN** its source URL, refresh metadata, and refresh, archive, restore, and separately confirmed deletion actions are available without changing their scope

#### Scenario: Preserve hierarchy preferences
- **WHEN** the operator returns to a feed or reloads it
- **THEN** only years explicitly expanded in the existing browser preference are open

#### Scenario: Use source controls on a narrow screen
- **WHEN** the operator views or manages source folders on a narrow screen
- **THEN** year controls, month rows, source metadata, and actions wrap or stack without overlap, clipped status text, or horizontal page overflow
