# Connected page design review

Status: All page designs approved on 2026-11-08. Feeds list version 3 and separate detail version 1 supersede the earlier master-detail layout. Approved rules are in DESIGN.md.

[Open the shared canvas](https://superdesign.dev/teams/ca1be67e-af28-40ed-9861-534950838927/projects/cc9adea7-1949-4c43-b5dc-7f46adfd97a1).

| Page | Preview | Application route |
| --- | --- | --- |
| Feeds — Simple list | [Open version 3](https://p.superdesign.dev/draft/f4af5ba3-6858-4482-bc91-62d3b825eee0) | `/manage/feeds` |
| Create a feed — Option A | [Open version 2](https://p.superdesign.dev/draft/a954b48e-cb39-4fab-947c-4279ee701efe) | `/manage/feeds/new` |
| PDF review — Option A | [Open version 2](https://p.superdesign.dev/draft/9ef922b6-59aa-4221-a321-b9bfffc130d4) | `/manage/pdf-review` |

## What to review

Feeds is now a simple Active/Archived list. Open a feed to reach its separate detail screen with RSS address, collapsed year rows, and contextual source management. Creation, editing and month forms are not embedded in the list. The previous crowded master-detail composition is superseded.

Create a feed keeps the existing Author name, optional Description and optional Refresh interval in hours. It has a constrained form width, inline validation, a focused error summary, pending button state, and Cancel. Submit only demonstrates validation; it does not create a feed.

PDF review keeps feed/year/month context above document cards with separate extraction and classification labels, publication decision selectors and document-specific source-link intent. The sample records show successful extraction, promotion classification and failed extraction with retry. Changes are local preview choices and never report actual server persistence.

All three retain Option A's Geist font, original Command mark, light zinc surfaces, blue accent, common shell, keyboard focus styles, mobile modal navigation, responsive layouts and shared desktop collapse preference. Cross-page sidebar and creation/cancel links navigate between generated previews. Sample data is labelled.

## Verification and limits

Retrieved generated HTML and corrected deterministic issues without further generation: initial year expansion, preview-only feedback, missing form labels and anchor IDs, mobile control sizing, mobile hierarchy form overflow risk, and inconsistent primary action color. Parsed inline JavaScript with Node. Superdesign accepted each corrected document as version 2 without warnings. Saved markup is checked after retrieval; no rendered browser validation was run.

These are visual prototypes, not complete workflow implementations. Some scope actions and source links only illustrate their intended behavior; dependent source selection, save failures, loading, empty states and all archive/restore cases still need implementation and verification against the existing application. No preview calls production APIs. Source context was explicitly approved for export by the user.

## Approval and next step

The user approved Create a feed and PDF review version 2, then the simplified Feeds list version 3 and separate detail version 1. Their rules are consolidated in docs/design/DESIGN.md. Production code and OpenSpec artifacts remain untouched; implementation is a separate task.

## Feeds simplification revision

[Simple feed list, version 3](https://p.superdesign.dev/draft/f4af5ba3-6858-4482-bc91-62d3b825eee0) links to [focused feed detail, version 1](https://p.superdesign.dev/draft/d677b336-17bc-4d93-9e1c-aa98bf9b6974). Both approved on 2026-11-08.

The user selected list-first browsing and confirmed the detail-screen structure. The list shows only feed identity, description/status and an Open feed link; Active/Archived tabs and Create feed are the primary controls. The separate detail page contains RSS with copy, Edit details, More actions and collapsed years. Connect month and source management open focused dialogs rather than displaying all fields and buttons at once. Refresh and archive actions are contextual. Preview data is explicitly illustrative; no backend calls occur.

Superdesign model generation was blocked by exhausted credits. These two drafts were authored directly from the approved shared shell and imported through Superdesign without generation credits. No approved creation/review draft was modified. Inline scripts parse, documents have one root with unique IDs, and import validation passed without warnings. The DOM parser does not support every modern CSS rule; this was a structural check only, not rendered visual verification.
