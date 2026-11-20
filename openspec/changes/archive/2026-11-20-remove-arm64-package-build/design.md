# Design

## Context

The container publishing workflow builds API, web, and extractor images. The API and web images are built as `linux/amd64`; the extractor's publish step is the only production build that also targets `linux/arm64`. The workflow sets up QEMU for multi-platform builds, and the extractor guide documents both architectures. See [proposal.md](proposal.md) and the `container-deployment` spec delta for the motivation and observable contract.

## Goals / Non-Goals

**Goals:**
- Keep all published image builds and release documentation aligned with the amd64-only support target.
- Remove setup used only for cross-architecture container builds.

**Non-Goals:**
- Remove arm64-specific dependency packages from lockfiles; these describe upstream platform variants and do not create project release builds.
- Change local development or deployment behavior beyond published image architecture availability.

## Decisions

- Set the extractor publish step to `linux/amd64`, matching the existing API and web build targets and the updated deployment requirement.
- Remove the QEMU setup step from the publish workflow because no release job will need emulation for a second architecture.
- Update the extractor CLI guide to describe its published image as amd64-only. Leave historical archived change artifacts intact; the active deployment spec and current user documentation describe supported behavior.
- Keep Buildx and the existing image scan/build steps, since they remain part of the amd64 release workflow.

## Risks / Trade-offs

- Hosts that require arm64 images can no longer run the published extractor image → state the amd64-only target in the deployment spec and extractor guide.
- Stale architecture claims could cause operators to expect arm64 support → update current documentation and verify no active release configuration requests an arm64 target.

## Migration Plan

Publish the next image tags with the extractor's amd64-only manifest. Existing tags and already-pulled arm64 images are not rewritten; operators on arm64 hosts must continue using an existing compatible image or move to an amd64 host. Rollback restores the multi-platform extractor publish target and QEMU setup.
