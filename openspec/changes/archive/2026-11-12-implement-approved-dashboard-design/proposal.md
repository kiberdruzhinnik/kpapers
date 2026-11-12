# Proposal

## Why

The approved Operator Overview design and five management screens in `docs/design/DESIGN.md` have not been implemented. The current neutral dashboard exposes a master-detail feed layout and persistent forms that obscure the primary operator tasks.

## What Changes

- Apply the approved light theme, blue semantic tokens, Geist typography, shell dimensions, responsive spacing, and accessible controls.
- Replace first-feed onboarding for established workspaces with publishing shortcuts and an honest review queue.
- Separate the feed list from feed detail, with Active/Archived organization, stable RSS identity, and contextual actions.
- Move editing, adding years, connecting months, and source management into focused accessible dialogs/disclosures.
- Align creation validation and PDF review with the approved screens while preserving routes, data, scoped operations, and local preferences.
- Add component and responsive browser coverage; run frontend type checking and tests.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `feed-management`: Approved dashboard presentation, separate list/detail views, focused management controls, accessible form errors, and distinct PDF classification and publication decisions.

## Impact

Frontend React components, Tailwind tokens, existing shadcn/ui composition, and frontend tests. Existing API contracts, scheduler behavior, public RSS URLs, management deep links, stored timestamps, and browser preference keys remain unchanged. Add shadcn primitives only where existing primitives cannot provide the required accessible interaction.
