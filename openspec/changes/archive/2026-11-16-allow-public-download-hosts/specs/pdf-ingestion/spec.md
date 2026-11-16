# Spec Delta

## MODIFIED Requirements

### Requirement: Validate provider PDF download destinations
Before fetching a discovered PDF, the system SHALL require an HTTPS download URL whose resolved destination addresses are public, and SHALL reject private or otherwise non-public network destinations without making the request. The same policy SHALL apply to every redirect destination.

#### Scenario: PDF download uses a public HTTPS host outside the provider's usual domains
- **WHEN** a provider returns an HTTPS download URL on a public network host
- **THEN** the system may request that destination subject to the existing timeout, response-size, and redirect controls

#### Scenario: Yandex Disk returns an allowed download destination
- **WHEN** a Yandex Disk PDF is fetched from an HTTPS destination that resolves to public addresses
- **THEN** the system may request that destination subject to the existing timeout, response-size, and redirect controls

#### Scenario: Yandex Disk returns a destination outside its allowlist
- **WHEN** a Yandex Disk PDF download URL uses a public HTTPS host outside the former provider allowlist
- **THEN** the system may request that destination subject to the same public-address, timeout, response-size, and redirect controls

#### Scenario: Mail Cloud returns an allowed download destination
- **WHEN** a Mail Cloud PDF is fetched from an HTTPS destination that resolves to public addresses
- **THEN** the system may request that destination subject to the existing timeout, response-size, and redirect controls

#### Scenario: Mail Cloud returns a destination outside its allowlist
- **WHEN** a Mail Cloud PDF download URL uses a public HTTPS host outside the former provider allowlist
- **THEN** the system may request that destination subject to the same public-address, timeout, response-size, and redirect controls

#### Scenario: Download URL resolves to a private or otherwise non-public address
- **WHEN** a provider returns an HTTPS download URL whose destination resolves to a private or otherwise non-public address
- **THEN** the system rejects the URL before connecting and records the document extraction as failed

#### Scenario: Redirect targets a private or otherwise non-public address
- **WHEN** a download redirects to a destination that resolves to a private or otherwise non-public address
- **THEN** the system stops the redirect and records the document extraction as failed

#### Scenario: Download destination uses a non-HTTPS scheme
- **WHEN** a provider returns a PDF download URL that does not use HTTPS
- **THEN** the system rejects the URL before making the download request and records the extraction as failed

#### Scenario: Download destination is rejected during refresh
- **WHEN** a provider returns an invalid PDF download destination for a document with previously cached content
- **THEN** the system records the refresh failure and retains the last successfully processed cached entry

## ADDED Requirements

### Requirement: Preserve supported public folder input hosts
The system SHALL continue accepting configured public folder input URLs only on `cloud.mail.ru` and `disk.yandex.ru`, subject to their existing URL shape and validation rules. Download destination host policy SHALL NOT broaden input URL validation.

#### Scenario: Configure a Mail Cloud public folder
- **WHEN** an operator provides a supported HTTPS public folder URL on `cloud.mail.ru`
- **THEN** the system accepts it using the existing Mail Cloud path and validation rules

#### Scenario: Configure a Yandex Disk public folder
- **WHEN** an operator provides a supported HTTPS public folder URL on `disk.yandex.ru`
- **THEN** the system accepts it using the existing Yandex Disk path and validation rules

#### Scenario: Configure a public folder URL on another host
- **WHEN** an operator provides a public folder URL on a host other than `cloud.mail.ru` or `disk.yandex.ru`
- **THEN** the system rejects it even if that host is publicly reachable

### Requirement: Report actionable errors for failed PDFs
When processing a PDF fails, the system SHALL retain and show an actionable error for that specific file, including its filename and the processing stage or concrete failure reason, without exposing credentials or ephemeral download secrets.

#### Scenario: A PDF fails during scheduled refresh
- **WHEN** processing a discovered PDF fails during a scheduled refresh
- **THEN** the refresh result identifies the filename and its specific failure reason instead of reporting only a generic PDF extraction failure

#### Scenario: An operator reviews a failed PDF
- **WHEN** an operator opens the PDF review queue for a failed document
- **THEN** the document displays its filename and actionable stored failure reason

#### Scenario: A failed PDF is retried successfully
- **WHEN** a retry successfully processes a previously failed PDF
- **THEN** the stored failure reason is cleared and the document is no longer reported as needing attention for that failure
