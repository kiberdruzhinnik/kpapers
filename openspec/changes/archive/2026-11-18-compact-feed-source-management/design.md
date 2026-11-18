# Design

## Context

See `proposal.md` for motivation and `specs/feed-management/spec.md` for user-visible requirements. Feed detail hierarchy is implemented in `HierarchyManagement.tsx`: year expansion is stored per feed/year, and each month already has a source-details disclosure and scoped actions. The approved Superdesign feed detail version 3 is the visual reference. The shared shell, proxy settings, and feed list are outside this change.

## Goals / Non-Goals

**Goals:**
- Apply the approved compact hierarchy to the existing React implementation without changing its data flow or action semantics.
- Keep year and month status visible at scan time while making URLs, timestamps, and operational controls available on demand.
- Retain keyboard access, screen-reader state, responsive wrapping, and the existing local year preference.

**Non-Goals:**
- Change APIs, stored feed/year/month data, refresh scheduling, RSS identity, proxy routing, or PDF review.
- Change the Feeds list or the overall feed detail shell.

## Decisions

- Keep the current `HierarchyManagement` component and API calls. Restyle its existing section and year groups rather than introducing a new hierarchy component; this avoids duplicating loading, archive, restore, and confirmation behavior.
- Keep each year as one compact bordered group. The heading carries year, status, configured-month count, expansion control, and the existing contextual year actions. Expansion continues to use the current feed/year local-storage key.
- Render configured months as compact rows. Keep month name, active/archive state, latest result, Review PDFs, and a source-details disclosure in the row. Use restrained separators and spacing instead of a second full-size card treatment.
- Keep source URL, local refresh timestamps, latest result/error, fetch and refresh actions, archive/restore, and confirmed deletion in the disclosure. Keep destructive deletion visually distinct and preserve its existing confirmation dialog.
- Use native React buttons with `aria-controls` and `aria-expanded` for disclosures. Preserve the existing year control's accessible name/state and the current focus-restoration behavior for confirmation dialogs; do not copy Superdesign's preview-only inline scripts or mock interactions.
- Update the approved feed detail reference in `docs/design/DESIGN.md` to link version 3 and describe the compact hierarchy. No migration is required; rollback is a UI-only revert.

## Risks / Trade-offs

- Hiding source details reduces information density when collapsed → Keep the latest result visible in the row and expose all remaining metadata through a clearly named disclosure.
- Long URLs or multiple actions can crowd narrow layouts → Allow rows and action groups to wrap, preserve URL word breaking, and retain mobile control sizing.
- Renaming the disclosure can invalidate existing test queries → Update focused component tests to assert the new label, expanded state, and retained actions.
