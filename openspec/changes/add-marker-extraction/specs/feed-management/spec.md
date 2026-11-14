# Spec Delta

## ADDED Requirements

### Requirement: Configure feed extraction engine
The system SHALL let the operator set each feed's extraction engine to inherit the global default, Pdfium, or Marker, and SHALL display the saved selection when editing the feed.

#### Scenario: Feed inherits the default
- **WHEN** a feed is configured to inherit
- **THEN** its effective engine follows the global default

#### Scenario: Feed overrides the default
- **WHEN** an operator selects an engine for a feed
- **THEN** that engine is used for the feed regardless of the global default

#### Scenario: Existing feed is migrated
- **WHEN** an existing feed has no engine override
- **THEN** it inherits the global default

#### Scenario: Invalid engine is submitted
- **WHEN** a feed is saved with an unsupported engine value
- **THEN** the system rejects the update and retains its previous configuration
