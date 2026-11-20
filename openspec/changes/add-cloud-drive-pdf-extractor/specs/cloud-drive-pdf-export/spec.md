# Spec Delta

## Purpose

Lets a user export readable Markdown from PDFs stored directly in a supported public Mail Cloud or Yandex Disk folder, without operating the web application.

## ADDED Requirements

### Requirement: Accept supported public folder links and an output directory
The CLI SHALL accept a public folder URL on `cloud.mail.ru` or `disk.yandex.ru` and a destination directory for Markdown output.

#### Scenario: Run with a Mail Cloud folder
- **WHEN** the user supplies a valid supported Mail Cloud public folder URL and an output directory
- **THEN** the CLI lists and processes eligible PDFs from that folder

#### Scenario: Run with a Yandex Disk folder
- **WHEN** the user supplies a valid supported Yandex Disk public folder URL and an output directory
- **THEN** the CLI lists and processes eligible PDFs from that folder

#### Scenario: Reject an unsupported or malformed URL
- **WHEN** the user supplies a URL that does not match a supported public folder format
- **THEN** the CLI reports an actionable input error and does not fetch files

### Requirement: Process only direct child PDFs
The CLI SHALL process PDF files directly inside the shared folder and SHALL NOT traverse subfolders.

#### Scenario: Folder contains direct PDFs and a subfolder
- **WHEN** the provider folder contains PDF files and a subfolder
- **THEN** the CLI processes the direct PDF files and ignores the subfolder and its contents

#### Scenario: Folder contains non-PDF files
- **WHEN** the provider folder contains direct non-PDF files
- **THEN** the CLI ignores those files

### Requirement: Write one Markdown extraction per PDF
For each successfully downloaded PDF, the CLI SHALL write a Markdown file in the destination directory using the existing PDF text cleanup and formatting behavior.

#### Scenario: Extract a text PDF
- **WHEN** a direct child PDF contains usable extractable text
- **THEN** the CLI writes its cleaned text as a Markdown file in the destination directory

#### Scenario: PDF has no usable text
- **WHEN** a PDF contains no usable native text or cannot be extracted
- **THEN** the CLI reports that file as failed and does not write an empty Markdown file for it

#### Scenario: Output names collide or contain path separators
- **WHEN** provider filenames would collide or contain characters that could address paths outside the destination directory
- **THEN** the CLI writes collision-safe Markdown filenames that remain within the destination directory

### Requirement: Create the output directory when needed
The CLI SHALL create the requested destination directory, including missing parent directories, before writing Markdown files.

#### Scenario: Output directory does not exist
- **WHEN** the requested destination directory is absent and can be created
- **THEN** the CLI creates it and continues processing

#### Scenario: Output path cannot be created
- **WHEN** the destination path cannot be created or is not writable
- **THEN** the CLI reports the filesystem error and exits unsuccessfully

### Requirement: Continue processing after an individual PDF failure
The CLI SHALL report failures per file, continue processing other listed PDFs, and exit unsuccessfully if any file fails.

#### Scenario: One PDF fails among several
- **WHEN** a listed PDF fails to download or extract while other PDFs remain
- **THEN** the CLI reports the failed filename and reason, processes the remaining PDFs, and returns a failure exit status

#### Scenario: All listed PDFs process successfully
- **WHEN** every listed PDF downloads and extracts successfully
- **THEN** the CLI reports successful completion and exits successfully

### Requirement: Protect provider downloads
The CLI SHALL use the existing safe download policy for provider files, requiring HTTPS and public destination addresses and validating each redirect destination before connecting.

#### Scenario: Provider returns a private download destination
- **WHEN** a PDF download URL or redirect resolves to a private or otherwise non-public address
- **THEN** the CLI rejects that download and reports a per-file failure without connecting to that address

### Requirement: Run in a container with mounted output
The extractor container SHALL accept the same URL and output-directory inputs as the CLI and SHALL write results to a mounted output directory.

#### Scenario: Run the container with a mounted destination
- **WHEN** the container is given a supported folder URL, an output directory under a host-mounted path, and permissions to write there
- **THEN** it performs the export and the Markdown files are available on the host

#### Scenario: Container cannot write to the mounted destination
- **WHEN** the container process lacks write permission for the output directory
- **THEN** it reports the filesystem error and exits unsuccessfully
