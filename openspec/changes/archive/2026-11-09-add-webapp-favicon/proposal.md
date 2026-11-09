# Proposal

## Why

The webapp currently has no favicon, so browser tabs and bookmarks lack the Kpapers brand mark. Adding the existing Command symbol in its dark rounded square gives the app a recognizable browser identity.

## What Changes

- Add a favicon that matches the symbol and dark rounded-square treatment used by the in-app Kpapers logo.
- Declare the favicon in the webapp document so browsers can use it in tabs and bookmarks.

## Capabilities

### New Capabilities
- `web-app-branding`: Browser-facing identity for the Kpapers webapp, including its favicon.

### Modified Capabilities
- None.

## Impact

- Affects the webapp's static assets and `web/index.html` metadata.
- No API or dependency changes are expected.
