# rss-publishing Specification

## Purpose

Publishes each eligible processed PDF as an item in a standards-based RSS feed with a permanent channel identity and stable item identifiers for subscribers.

## Requirements

### Requirement: Publish one RSS item per eligible PDF
The system SHALL expose each eligible, successfully processed PDF as a separate item in its feed's RSS channel.

#### Scenario: Multiple PDFs are discovered
- **WHEN** a month folder contains multiple eligible PDFs
- **THEN** the RSS channel contains a distinct item for each successfully processed PDF

#### Scenario: Archived hierarchy is not published
- **WHEN** a feed, year, or month is archived
- **THEN** RSS omits entries under that archived scope until it is restored

#### Scenario: Promotional PDF is excluded
- **WHEN** a PDF is classified promotional and not overridden by the operator
- **THEN** the PDF is absent from the RSS channel

### Requirement: Keep RSS channel identity stable
The system SHALL use the feed's persistent ID as the basis for its RSS self URL and channel identity, independent of editable display metadata.

#### Scenario: Feed is renamed
- **WHEN** the operator renames a feed
- **THEN** its RSS identity remains unchanged and existing subscribers continue using the same feed URL

### Requirement: Keep RSS item identifiers stable
The system SHALL assign a stable non-permalink GUID to each source PDF within a feed and preserve that GUID when the same source file is refreshed.

#### Scenario: Unchanged PDF is refreshed
- **WHEN** the same source PDF is seen again with unchanged content
- **THEN** its RSS item retains the same GUID and is not presented as a new item

#### Scenario: PDF content is updated
- **WHEN** a source PDF changes but remains the same provider file
- **THEN** its RSS item keeps its GUID and its cached content and publication metadata are updated

### Requirement: Include safe extracted content in RSS
The system SHALL include sanitized extracted HTML as the RSS item's description or content.

#### Scenario: Subscriber reads an article
- **WHEN** an RSS reader retrieves a feed containing a processed PDF
- **THEN** the item exposes its safe HTML content

### Requirement: Link RSS items to exact PDF downloads
Each RSS item SHALL link to a stable application URL keyed to its source document. That URL SHALL resolve the exact source file and redirect to a fresh provider-issued direct PDF download URL. It SHALL NOT point to a containing cloud folder or a cloud-drive file-view page, and the application SHALL NOT proxy or republish the PDF bytes.

#### Scenario: Follow an RSS item link
- **WHEN** a reader follows an RSS item link
- **THEN** the application redirects to the direct download for that exact PDF

#### Scenario: A folder contains multiple PDFs
- **WHEN** multiple PDFs from one cloud folder are published in the same RSS channel
- **THEN** each item has its own stable application link, and following each link redirects to that item's direct PDF download rather than the shared containing folder

#### Scenario: Mail Cloud public-folder entry is not itself a file download
- **WHEN** a Mail Cloud folder listing returns a file `weblink` that identifies a folder or viewer page, including the file `2026-11-06 [skladchik.org].pdf` in the folder `https://cloud.mail.ru/public/Ytr8/b4fynf4PP`
- **THEN** the RSS item link redirects to the provider-issued direct download for the root-relative file path `Ytr8/b4fynf4PP/2026-11-06 [skladchik.org].pdf`, not to the folder or viewer page

#### Scenario: Yandex Disk path query is a viewer page
- **WHEN** a Yandex Disk file is represented by a folder URL with a `path` query, such as `https://disk.yandex.ru/d/iCbUDNZAMJYppQ?path=%2F20261105%20-%20%D0%AF%D0%B4%D0%B5%D1%80%D0%BD%D1%8B%D0%B9%20%D0%9B%D0%B8%D0%B2%D0%B0%D0%BD.%20%D0%9F%D1%83%D1%82%D0%B5%D0%B2%D0%BE%D0%B4%D0%B8%D1%82%D0%B5%D0%BB%D1%8C%20%D0%BF%D0%BE%20%D0%B2%D1%8B%D0%B1%D0%BE%D1%80%D0%B0%D0%BC%20%D0%B8%20%D0%B1%D1%83%D0%B4%D1%83%D1%89%D0%B5%D0%BC%D1%83%20%D0%A1%D0%A8%D0%90.pdf`
- **THEN** the RSS item link redirects to the provider-issued direct download for that exact PDF, not to the Yandex Disk viewer page

### Requirement: Support end-to-end validation with Miniflux
The system SHALL be verifiable through an isolated end-to-end test in which Miniflux subscribes to a public feed and ingests its published PDF items.

#### Scenario: Miniflux subscribes to a published feed
- **WHEN** the end-to-end test creates a feed, links a fixture month containing eligible PDFs, and subscribes Miniflux to that feed's stable RSS URL
- **THEN** Miniflux discovers the channel and displays one entry per eligible PDF with its safe content and a stable application link that redirects to that exact PDF's download

#### Scenario: Refresh preserves item identity in Miniflux
- **WHEN** the same provider file is refreshed without a content change, or its content changes while retaining its provider identity
- **THEN** the RSS GUID remains stable and Miniflux does not create a duplicate entry for that PDF

#### Scenario: Test environment is isolated
- **WHEN** the Miniflux end-to-end test environment is started or cleaned up
- **THEN** it uses disposable test configuration and data without changing the normal application or quickstart deployment
