# Spec Delta

## Purpose

Provides operators with API operational events through the container's standard output stream, where the container platform can collect and display them.

## ADDED Requirements

### Requirement: API operational events are written to stdout
The API SHALL emit startup, HTTP request, and background refresh processing events to stdout so container log collection can expose them.

#### Scenario: API starts successfully
- **WHEN** the API completes initialization and begins serving requests
- **THEN** it writes a startup event to stdout

#### Scenario: API initialization fails
- **WHEN** API initialization fails
- **THEN** it writes a structured error event to stdout, exits with a nonzero status, and does not duplicate the error as plain text on stderr

#### Scenario: API utility command fails
- **WHEN** the `--healthcheck` or `--hash-password` command fails
- **THEN** it exits with a nonzero status and emits a useful diagnostic

#### Scenario: API handles an HTTP request
- **WHEN** the API finishes handling an HTTP request
- **THEN** it writes a request event to stdout with the method, path, response status, and request duration

#### Scenario: Background refresh work fails
- **WHEN** the scheduler fails to enqueue due work or process a refresh job
- **THEN** it writes an error event to stdout with enough context to identify the failed operation

### Requirement: API logs do not expose sensitive values
The API SHALL NOT include passwords, session credentials, cookies, signing keys, or provider access credentials in its log output.

#### Scenario: Request contains credentials
- **WHEN** a request includes authentication credentials or session cookies
- **THEN** the corresponding request event omits those values

#### Scenario: Provider operation fails
- **WHEN** a provider operation fails with a response or URL containing sensitive data
- **THEN** the error event omits credentials and secret-bearing URL components
