# Tasks

## 1. Configure the web image

- [x] 1.1 Replace the hardcoded API origin in the Nginx config with an `API_URL` template and verify all API proxy locations use it while retaining existing paths.
- [x] 1.2 Add web-container startup validation for required, valid HTTP(S) origin `API_URL`; verify missing and malformed values fail with clear errors before Nginx starts.
- [x] 1.3 Add focused checks for configured upstream rendering and startup failures, and verify the web image build includes the template and validation behavior.

## 2. Update deployment configuration and guidance

- [x] 2.1 Pass `API_URL` from Compose environment interpolation into the web service and verify rendered Compose configuration includes the value.
- [x] 2.2 Add `API_URL=http://api:8080` to the TrueNAS environment example and document matching the hostname to the API service name; verify the documented deployment settings agree.

## 3. Verify container integration

- [x] 3.1 Build and start the Compose stack with a non-default API service hostname and verify the web container resolves the configured upstream and serves proxied API routes.
