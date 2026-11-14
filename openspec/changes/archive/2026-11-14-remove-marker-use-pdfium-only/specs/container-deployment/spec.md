# Spec Delta

## MODIFIED Requirements

### Requirement: Publish application container images
The project SHALL publish publicly pullable API and web container images to GitHub Container Registry from the repository's GitHub Actions workflow.

#### Scenario: Build from the default branch
- **WHEN** a commit is pushed to the default branch
- **THEN** the workflow builds and publishes the API and web images with a stable default-branch tag and a commit-specific tag

#### Scenario: Build from a version tag
- **WHEN** a version tag is pushed
- **THEN** the workflow publishes the API and web images with that version tag

#### Scenario: Pull published images without authentication
- **WHEN** a deployment host pulls a published API or web image from GHCR
- **THEN** the image is publicly accessible without registry credentials

## ADDED Requirements

### Requirement: Provide a pdf_oxide TrueNAS Compose deployment example
The project SHALL provide a Compose example that deploys only the API and web images from GHCR and loads operator configuration from a colocated `.env` file. The web service SHALL join the stack's default network and an externally managed `npm` network. The API SHALL extract PDF text with `pdf_oxide` in-process, without a Marker worker or Pdfium native runtime.

#### Scenario: Start the deployment
- **WHEN** an operator supplies the required values in `.env` and starts the Compose project
- **THEN** Compose pulls the published API and web images, starts the web service after the API is healthy, and connects the web service to the `npm` network

#### Scenario: Preserve application data
- **WHEN** the Compose project is stopped, recreated, or its images are updated
- **THEN** the API database remains stored under `./kpapers_data` mounted at `/data`, and the `.env` configuration remains available to the API

#### Scenario: Connect web traffic through Nginx Proxy Manager
- **WHEN** the web service starts
- **THEN** it joins both the Compose default network and the externally managed `npm` network so Nginx Proxy Manager can reach it by service name

#### Scenario: Extract PDFs with pdf_oxide
- **WHEN** an operator starts the TrueNAS deployment with its default configuration
- **THEN** the API extracts eligible PDFs with `pdf_oxide` without a separate extraction worker or Pdfium native runtime

#### Scenario: Write data using the TrueNAS app identity
- **WHEN** the API image runs with a TrueNAS dataset or volume configured for the conventional `app` UID:GID `568:568`
- **THEN** the API process can read and write its database under `/data` as UID:GID `568:568`

#### Scenario: Configure operator credentials
- **WHEN** an operator configures the deployment
- **THEN** the example identifies the required operator password hash and session signing key settings and keeps their values outside the Compose file

#### Scenario: Update the deployment
- **WHEN** an operator requests the latest published images and recreates the services
- **THEN** Compose updates the running deployment while retaining the `./kpapers_data` database and `.env` configuration

## REMOVED Requirements

### Requirement: Provide a TrueNAS Compose deployment example
**Reason**: The existing requirement mandates a Marker worker and model cache, which are removed.
**Migration**: Use the replacement `pdf_oxide` TrueNAS Compose requirement. Keep the `./kpapers_data` bind mount, `.env` configuration, TrueNAS app identity, API health dependency, and external `npm` network.
