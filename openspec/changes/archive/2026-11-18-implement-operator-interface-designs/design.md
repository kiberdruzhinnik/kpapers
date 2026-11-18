# Design

## Context

See `proposal.md` for motivation and approved reference versions. The application already implements the relevant interactions in separate React components: PDF review, global proxy settings, feed list/detail, per-feed proxy settings, and source hierarchy. The shared dashboard shell, route handling, API, and persistence are outside the visual redesign. Existing specifications and tests define the behavior to preserve.

The Superdesign drafts are visual references, not production source. Their sample data, preview navigation, remote scripts, and simulated actions must not be copied into the application.

## Goals / Non-Goals

**Goals:**

- Bring the four screens into alignment with their approved current drafts while keeping the current application shell and real data.
- Make PDF review status and decisions, proxy precedence, and feed/source hierarchy scannable at desktop and narrow widths.
- Retain existing accessible controls and established interaction states.

**Non-Goals:**

- Change backend APIs, proxy resolution, document eligibility, feed identity, routes, saved preferences, or destructive-action behavior.
- Redesign Overview, Create a feed, sign-in, or the shared navigation shell.
- Add packages or introduce a new component system.

## Decisions

### Treat the approved drafts as layout and visual references

Implement the drafts using the current React components and shared UI primitives. Preserve the 240px/72px desktop sidebar, 64px header, typography, color tokens, and breakpoints in `docs/design/DESIGN.md`. The current dashboard shell already provides this foundation, so page changes should stay within their content areas unless a concrete mismatch is discovered.

Alternative considered: transplanting generated HTML from Superdesign. Rejected because the drafts contain illustrative data and preview-only JavaScript, and depend on CDN Tailwind and Iconify rather than the application’s component and routing stack.

### Keep each screen's existing state model and controls

- **PDF review:** Restyle the mode switch, source selectors, queue header, and document rows. Keep Needs review, Archived, and Browse by source; feed/year/month deep links; loading, error, and empty states; source-PDF links; extraction retry; independent classification and publication decision; and per-row save feedback. Use real queue counts/data only where already available.
- **Settings:** Separate the global proxy configuration and its current state from a concise routing-precedence panel. Preserve the password input, write-only credentials, validation, save/replace/clear actions, API error messaging, and the existing order: global GUI proxy, environment fallback, then direct connection.
- **Feeds list:** Keep the Active/Archived tabs, list rows, Create feed action, archived restore/delete menu, and empty states. Apply the current row density and hierarchy without introducing search, counts, or other unsupported data.
- **Feed detail:** Retain the All feeds route action, stable RSS address/copy feedback, Edit details, scoped More actions, per-feed proxy settings, and source-folder hierarchy. Keep years collapsed by default unless the existing per-feed/year preference says otherwise; keep Connect month and Manage source focused in their existing dialogs/disclosures.

Alternative considered: consolidating global and feed-specific proxy settings into one form. Rejected because they have distinct scope and precedence, and both current designs make that distinction explicit.

### Use existing primitives and semantic status treatment

Use the existing Button, Card, Badge, Field, Select, Dialog, DropdownMenu, AlertDialog, Empty, Alert, and Sonner components as appropriate. Keep headings, labels, status text, and accessible names semantic. Status meaning must remain readable without color alone. Preserve visible focus, keyboard behavior, reduced-motion support, and mobile target sizing from the approved design guide.

Alternative considered: adding new UI dependencies or page-specific CSS frameworks. Rejected because the current stack covers the layouts and interactions and adding another system would increase styling and accessibility drift.

## Risks / Trade-offs

- [Visual changes can hide important controls or status] → Keep every existing control/state in its page and verify queue, form, error, empty, archived, and destructive-action views.
- [Generated preview content can be mistaken for real data or behavior] → Use Superdesign only as a reference; bind all production content and actions to current application state and APIs.
- [Narrow layouts can crowd selectors and row actions] → Stack the relevant groups below the existing responsive breakpoints, preserve wrapping for long names/URLs, and review at 320, 375, 768, 1024, and 1440px plus landscape.
- [Proxy redesign could expose saved credentials] → Keep proxy inputs write-only and render only the endpoint metadata already returned by the API.

## Migration Plan

No data or API migration is required. Implement page changes incrementally and retain existing interaction tests. Reverting the UI commits restores the current presentation without changing persisted data or routes.
