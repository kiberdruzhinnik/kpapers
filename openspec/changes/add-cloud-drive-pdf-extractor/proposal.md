# Proposal

## Why

People who receive public Mail Cloud or Yandex Disk folders need a local, repeatable way to collect the PDFs and read their extracted text without setting up the full web application. A small CLI can reuse the existing provider and PDF-processing behavior while working both on a host and in Docker.

## What Changes

- Add a standalone CLI command that accepts a supported Mail Cloud or Yandex Disk public folder URL and an output directory.
- List only direct child PDFs, download each PDF, and write a Markdown extraction for each file using the existing PDF cleanup and extraction pipeline. Create the output directory when needed.
- Provide a Docker image for running the extractor and publish it for `linux/amd64` and `linux/arm64` through the container release workflow.

## Capabilities

### New Capabilities

- `cloud-drive-pdf-export`: Export text from direct child PDFs in supported public cloud folders to local Markdown files.

### Modified Capabilities

- `container-deployment`: Publish a standalone extractor container for both supported CPU architectures, alongside existing application images.

## Impact

- Rust CLI argument handling and orchestration; existing Mail Cloud and Yandex Disk provider adapters; existing `pdf_processing` extraction and cleanup.
- A new Docker build target or Dockerfile and the GHCR container publishing workflow.
- Extractor usage documentation. No database or web API behavior changes are required.
