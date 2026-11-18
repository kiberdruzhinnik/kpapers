# Kpapers dashboard design rules

Status: Approved visual direction — Option A, Operator Overview. Selected by the user on 2026-11-08: “i like A”. Approval covers the dashboard design direction and the rules below; application implementation remains a separate step.

## Approved reference and scope

- [Option A preview, version 2](https://p.superdesign.dev/draft/0d73a8e3-facf-4122-b8e2-a561e1812e08).
- [Superdesign canvas](https://superdesign.dev/teams/ca1be67e-af28-40ed-9861-534950838927/projects/cc9adea7-1949-4c43-b5dc-7f46adfd97a1).
- Stack: React, Tailwind CSS v4, existing shadcn/ui components backed by Radix.
- Light theme, restrained blue accent, Geist Variable typography, Lucide outline icons.
- This document is the approved design reference. PROPOSAL.md retains the alternatives and research history; Option B is not selected.

## Layout and content hierarchy

Use a 240px expanded desktop sidebar, 72px collapsed sidebar, 64px sticky header and centered content area with a 1200px maximum width. Retain the Command mark in its black rounded square and the Kpapers wordmark. Brand navigation returns to `/manage/`. Sidebar items are Overview, Feeds and PDF review; indicate the active route with blue text and a pale blue surface. Omit the empty Your feeds subsection unless actual feed links are available.

The header presents the current location, operator identity and sign-out. The page starts with a concise Overview heading and description, with Create feed as the primary action linking to `/manage/feeds/new`.

Show three equally sized metric cards, about 140px high with flexible height for wrapping text: Active feeds, Source folders and Needs review. Each contains its label, tabular value, restrained icon and short explanation. Screenshot values 3 / 3 / 0 are sample data only.

Below the metrics, use a two-column region in a roughly 2:1 ratio on wide screens:

- Publishing workflow: Manage feeds, Connect source folders and Review PDFs shortcuts. Manage feeds and Connect source folders lead to `/manage/feeds`; Review PDFs leads to `/manage/pdf-review`.
- Review queue: current attention state and Open PDF review action. For a confirmed zero count, use “No documents need attention” and optionally “The review queue is empty”. For nonzero counts, present the count and a clear review action. For unavailable data, show an unavailable state rather than success.

Replace first-feed onboarding when active feeds already exist. Keep the zero-feed state useful with a Create feed action. Do not infer source synchronization health, publication completion or setup progress from aggregate counts.

## Typography

| Role | Size / line height | Weight |
| --- | --- | --- |
| Page title | 28 / 36 px | 600 |
| Mobile page title | 24 / 32 px | 600 |
| Section title | 16 / 24 px | 600 |
| Body and controls | 14 / 20 px | 400 / 500 |
| Secondary metadata | 12 / 18 px | 400 |
| Metrics | 30 / 36 px, tabular numerals | 600 |

Font: Geist Variable, system sans-serif fallback. Use 12px only for metadata; essential actions and form labels use 14px. Long feed names, source URLs and filenames wrap safely.

### Colors and semantic tokens

| Token | Proposed value | Use |
| --- | --- | --- |
| background | #FAFAFA | Workspace canvas |
| card / popover | #FFFFFF | Raised surfaces |
| foreground | #18181B | Primary text |
| muted | #F4F4F5 | Secondary surfaces |
| muted-foreground | #71717A | Secondary text on white |
| primary / ring | #2563EB | Primary action and focus |
| primary-foreground | #FFFFFF | Primary button label |
| accent | #EFF6FF | Selected navigation |
| accent-foreground | #1D4ED8 | Text on selected surface |
| border / input | #E4E4E7 | Decorative dividers and surfaces |
| success text / surface | #15803D / #F0FDF4 | Confirmed clear queue or success |
| warning text / surface | #B45309 / #FFFBEB | Attention and processing warnings |
| destructive | #DC2626 | Failed actions / permanent deletion |

Decorative borders need not carry control affordance alone. Use a darker control outline where needed for 3:1 non-text contrast. Validate text at 4.5:1 and focus/control indicators at 3:1 on their actual backgrounds; avoid muted text on tinted surfaces until checked. Status always includes readable text and an icon, not color alone.

### Spacing, shape and motion

4px base spacing. Use 8px within controls, 12px within compact groups, 16px between cards, 24px panel padding and 32px desktop page gutters. Mobile gutters 16px and panel padding 16px. Radius 8px controls and 12px cards; 1px borders; minimal shadows. Buttons 36px desktop, 44px mobile touch area. Icons 16px controls, 18px navigation. Hover/focus transitions 150ms; reduced-motion removes nonessential movement. No gradients, decorative charts, photos or large hero blocks.

### Component mapping

| Purpose | Existing / preferred shadcn component |
| --- | --- |
| Actions | Existing Button: default, outline, ghost, destructive |
| Summary and workflow panels | Existing Card family |
| Status | Existing Badge with explicit semantic variants |
| Form fields | Existing Input, Label, Textarea, Select |
| Mobile navigation | Existing Sheet; evaluate standard Sidebar/SidebarTrigger for future implementation |
| Navigation context | Breadcrumb pattern; add primitive only if implementation warrants it |
| Destructive confirmation | Existing AlertDialog |
| Feedback | Existing Sonner, inline errors, Skeleton |
| Feed hierarchy | Accessible collapsible year toggle; preserve browser-local expansion preference |

Do not install or replace components during this design phase. Reuse existing primitives before adding new ones. Keep form widths and action alignment consistent; distinguish month-linking form, source/status metadata, and action groups.

## Responsive behavior

- Below 640px: one column for metrics, workflow shortcuts and review panel; 16px page gutters; 16px card padding. Use a 24px/32px page title and at least 44px touch areas for controls.
- 640–767px: metrics may use two columns with Needs review spanning the row; workflow shortcuts may share a row if their labels fit. Lower panels stay stacked.
- From 768px: desktop sidebar is available and remembers collapse state. Below 768px, use the existing Sheet pattern with a named menu trigger, modal focus containment, Escape/close support and focus restoration.
- From 1024px: three equal metric columns and the lower 2:1 workflow/review split, provided text fits without crowding. Otherwise stack the lower panels.
- Page title, description and Create feed wrap naturally. Keep sign-out accessible on mobile even when its visible label is hidden.
- Long names, source URLs and filenames wrap without clipping. Do not hide essential controls or cause horizontal page overflow.

## Accessibility and interaction

Use semantic headings and navigation, aria-current for the active route, aria-expanded for sidebar/year toggles, and accessible names for every icon-only control. Decorative icons are hidden from assistive technology. Provide visible focus with a 2px blue outline and offset, with sufficient contrast on the actual adjacent surface. Keep logical keyboard order and return focus after dialogs close.

Use the existing application preference storage for sidebar collapse and feed/year expansion. Prototype storage keys and hash links are preview mechanics, not implementation contracts. Navigation must use the existing real application routes when implemented.

Normal text requires at least 4.5:1 contrast. Calculated proposed pairs: primary text on white 17.72:1; muted on white 4.83:1; muted on canvas 4.63:1; white on blue primary 5.17:1; selected text on blue-50 6.16:1; success 4.79:1; warning 4.84:1; white on destructive 4.83:1. Control/focus indicators require 3:1 where applicable. Color never carries status by itself.

## Behavioral constraints and states

- Preserve protected deep links, browser history, Create feed success/cancel, RSS identity and copy feedback.
- Active feeds count remains sourced from active feeds; source folders and review counts use their actual summary fields. Zero is a valid value; skeleton/loading, unavailable/error and empty data are separate states.
- Show refresh attempts, next scheduled times, and success/failure per month; never imply a global healthy scheduler from absent data. Browser-local timestamp formatting must preserve stored instants.
- Preserve archive, restore and separately confirmed permanent deletion at feed/year/month scopes.
- PDF review preserves feed/year/month selection, extraction state, publication override feedback, and document-specific source links.
- No invented feed names, activity timestamps, subscriber counts, trends, document totals, search or settings workflows.
- Keep scheduler promotional banners and footer slogans absent, per main specifications.

## Validation and implementation handoff

The selected Superdesign version was retrieved and its typography, responsive classes, Command mark, navigation controls, route annotations and corrected review text were checked in markup. No rendered browser validation or production implementation has been performed.

When implementation is authorized, reuse existing shadcn/ui React components and application state. Superdesign HTML is a visual reference: its CDN Tailwind runtime, Iconify elements, native dialog preview, hash links and illustrative sign-out must not replace the existing React/Radix workflows. Validate at 375, 768, 1024 and 1440px, plus 320px overflow and landscape; check keyboard navigation, reduced motion, loading/error states and actual data. Preserve the OpenSpec feed-management and refresh-scheduling contracts.

## Approved Create a feed page

Approved on 2026-11-08. [Create a feed, version 2](https://p.superdesign.dev/draft/a954b48e-cb39-4fab-947c-4279ee701efe).

Use the shared Option A shell with Feeds active and a Workspace / Feeds / Create a feed breadcrumb. Center a form of approximately 640px maximum width. Keep only the existing fields: required Author name, optional Description textarea and optional Refresh interval in hours. Explain that blank uses the current/older month defaults. Keep Cancel and blue Create feed together; stack on mobile. Link field errors to controls and focus a linked error summary after invalid submit. Preserve typed input on failure and show pending/disabled submission. Cancel returns to the feed list; real successful creation opens the created feed detail. Connect source folders after creation. The prototype must not create data or imply persistence.

## Approved PDF review page

The current design was approved on 2026-11-17. [PDF review, version 3](https://p.superdesign.dev/draft/9ef922b6-59aa-4221-a321-b9bfffc130d4) supersedes version 2 as the implementation reference.

Use the shared Option A shell with PDF review active. Keep Needs review, Archived and Browse by source modes easy to distinguish. In source-browsing mode, place labelled dependent Feed, Year and Month selectors above the discovered PDF list. Retain queue source context and partial/full review deep links. Each document row shows filename, extraction state, classification, document-specific source link and a labelled publication decision: Automatic, Include in feed or Exclude from feed. Keep classification and override distinct. Failed extraction shows an error and Retry extraction; Include cannot bypass successful extraction. Show pending save/error feedback per row and retain the prior choice after failure. Stack filters and actions on mobile; wrap filenames. Source links must identify individual documents, not folders. Prototype choices remain local and labelled as previews.

## Approved Settings page

Approved by the user on 2026-11-17. [Settings, version 4](https://p.superdesign.dev/draft/019c063d-9171-4d5e-a8cc-bb5cef44889d) is the implementation reference.

Use the shared Option A shell with Settings active. Present global fetch proxy configuration and its current configured/unconfigured state in a focused panel, with credential-safe replacement and clear actions. Show routing precedence separately and in order: global GUI proxy, `KPAPERS_PROXY_URL` environment fallback, then direct connection. Explain that per-feed Custom and Direct modes override inherited routing. Keep saved credentials write-only and preserve validation and request feedback.

## Approved Feeds list and detail screens

The current references were approved by the user on 2026-11-17. The feed list version 3 was previously approved on 2026-11-08; the earlier master-detail composition is superseded.

- [Feed list, version 3](https://p.superdesign.dev/draft/f4af5ba3-6858-4482-bc91-62d3b825eee0), application route `/manage/feeds`.
- [Feed detail, version 2](https://p.superdesign.dev/draft/d677b336-17bc-4d93-9e1c-aa98bf9b6974), application route `/manage/feeds/:feedId`.

### Feed list

Use a single list surface with restrained rows showing feed name, short description, status and an Open feed affordance. Active/Archived tabs organize the list; Create feed is the primary blue action. Opening a feed navigates to its own detail screen. Keep year/month hierarchy, editing forms and refresh controls off the list. Archived feeds retain Restore and separately confirmed permanent deletion through contextual controls. Preserve an actionable empty state when no feeds exist.

### Feed detail

Show All feeds as a back link, feed title/description and status, Edit details, and a compact More actions control. Put the stable RSS address with Copy link in a concise surface. Keep feed identity stable across edits and archive/restore.

Source folders appears as one clear section with Add year and simple year rows. Years start collapsed unless a stored preference explicitly expands them. Expanding a year reveals month rows; Connect month opens a focused form instead of exposing fields permanently. Month rows show the month, a concise state, Manage source and Review PDFs. Manage source reveals the folder URL, last attempt, next refresh, latest result and scoped actions. Keep feed/year/month refresh, archive, restore and permanent deletion available with their existing scope and confirmations; distinguish destructive actions in menus/dialogs.

Use focused dialogs/disclosures for Add year, Connect month and Edit details. Preserve existing field validation and server outcomes. Menus and dialogs must support keyboard navigation, Escape, labelled controls, focus containment/restoration and visible focus. In production, prefer existing shadcn/ui primitives over prototype native dialogs or generic details-based menus. On mobile, wrap actions, stack groups and retain 44px targets without horizontal overflow.

### Design handoff status

Overview, Feeds list, feed detail, Create a feed, PDF review and Settings are approved as visual references. Current implementation targets are Feeds list v3, Feed detail v2, PDF review v3 and Settings v4. Sample records, preview URLs and preview storage keys are illustrative; implementation uses actual data, application routes and existing preferences. Superdesign has no approval-status command, so approval is recorded here and in `.superdesign/resume.json`. Production code is unchanged. Approval does not claim complete prototype interactions or rendered browser verification.
