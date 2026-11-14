# Spec Delta

## ADDED Requirements

### Requirement: Validate provider PDF download destinations
Before fetching a discovered PDF, the system SHALL validate the initial download URL against the HTTPS host allowlist for the configured provider. URLs that do not satisfy the provider's scheme, port, credential, and host constraints SHALL be rejected without making the request.

#### Scenario: Yandex Disk returns an allowed download destination
- **WHEN** a Yandex Disk PDF is fetched from an HTTPS destination on the Yandex download host allowlist
- **THEN** the system may request that destination subject to the existing timeout, response-size, and redirect controls

#### Scenario: Yandex Disk returns a destination outside its allowlist
- **WHEN** a Yandex Disk PDF download URL uses an unapproved host, scheme, port, or embedded credentials
- **THEN** the system rejects the URL before making the download request and records the extraction as failed

#### Scenario: Mail Cloud returns an allowed download destination
- **WHEN** a Mail Cloud PDF is fetched from an HTTPS destination on the Mail Cloud download host allowlist
- **THEN** the system may request that destination subject to the existing timeout, response-size, and redirect controls

#### Scenario: Mail Cloud returns a destination outside its allowlist
- **WHEN** a Mail Cloud PDF download URL uses an unapproved host, scheme, port, or embedded credentials
- **THEN** the system rejects the URL before making the download request and records the extraction as failed

#### Scenario: Download destination is rejected during refresh
- **WHEN** a provider returns an invalid PDF download destination for a document with previously cached content
- **THEN** the system records the refresh failure and retains the last successfully processed cached entry
