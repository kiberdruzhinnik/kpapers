# Spec Delta

## Purpose

Provides a disposable local environment so an operator can quickly try the web application without configuring production credentials or storage.

## ADDED Requirements

### Requirement: Start an isolated local test environment
The system SHALL provide a quickstart script that generates non-production settings and starts the web and API services in a dedicated Compose project and data volume, binding the web endpoint to all host interfaces.

#### Scenario: Start quickstart for the first time
- **WHEN** an operator runs the quickstart script with Docker Compose and OpenSSL available
- **THEN** the script creates a private quickstart environment file, starts the services with a web endpoint available on all host interfaces, and prints the URL and generated operator login

#### Scenario: Production configuration already exists
- **WHEN** an operator runs quickstart beside a production `.env` or Compose deployment
- **THEN** quickstart uses its own environment file, project name, and data volume without changing the production configuration or data

### Requirement: Remove the isolated local test environment
The system SHALL provide a cleanup script that removes only quickstart resources and clearly reports that quickstart data will be deleted.

#### Scenario: Remove quickstart resources
- **WHEN** an operator runs the quickstart cleanup script
- **THEN** it removes the quickstart containers, network, named volume, local images, and generated environment file

#### Scenario: Protect a normal deployment
- **WHEN** cleanup is run while a normal Compose deployment or its environment file exists
- **THEN** cleanup leaves the normal deployment and its data untouched
