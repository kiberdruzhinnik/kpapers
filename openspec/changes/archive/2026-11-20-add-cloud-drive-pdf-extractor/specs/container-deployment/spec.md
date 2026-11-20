# Spec Delta

## MODIFIED Requirements

### Requirement: Publish application container images
The project SHALL publish publicly pullable API, web, and standalone PDF extractor container images to GitHub Container Registry from the repository's GitHub Actions workflow. The extractor image SHALL support both `linux/amd64` and `linux/arm64`.

#### Scenario: Build from the default branch
- **WHEN** a commit is pushed to the default branch
- **THEN** the workflow builds and publishes the API, web, and extractor images with a stable default-branch tag and a commit-specific tag

#### Scenario: Build from a version tag
- **WHEN** a version tag is pushed
- **THEN** the workflow publishes the API, web, and extractor images with that version tag

#### Scenario: Pull published images without authentication
- **WHEN** a deployment host pulls a published API, web, or extractor image from GHCR
- **THEN** the image is publicly accessible without registry credentials

#### Scenario: Pull the extractor image on amd64
- **WHEN** a host running `linux/amd64` pulls a published extractor image tag
- **THEN** the container registry provides an image that runs on that architecture

#### Scenario: Pull the extractor image on arm64
- **WHEN** a host running `linux/arm64` pulls a published extractor image tag
- **THEN** the container registry provides an image that runs on that architecture
