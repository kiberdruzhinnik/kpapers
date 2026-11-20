# Tasks

## 1. Standardize published image builds

- [x] 1.1 Remove QEMU setup and change extractor publication to `linux/amd64`; verify every application image build and publish target in the workflow is amd64-only.
- [x] 1.2 Update the extractor CLI guide to state amd64-only support; verify current deployment documentation no longer promises arm64 images.

## 2. Verify deployment contract

- [x] 2.1 Validate the OpenSpec change and inspect the publishing workflow to confirm it cannot build or publish an arm64 image variant.
