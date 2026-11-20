# Proposal

## Why

The project currently publishes the standalone PDF extractor for both amd64 and arm64, while the API and web images are amd64-only. Supporting multiple architectures adds build and release complexity; standardizing all published images on amd64 makes the supported release target clear.

## What Changes

- Remove arm64 from every container image build and release target.
- Document that published API, web, and extractor images support `linux/amd64` only.
- Remove the arm64 pull scenario from the deployment requirements and extractor usage documentation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `container-deployment`: require published application images to support `linux/amd64` only.

## Impact

The container publishing workflow, extractor deployment documentation, and `container-deployment` specification are affected. Existing API and web image targets are already amd64-only; the extractor image's arm64 variant will no longer be built or published.
