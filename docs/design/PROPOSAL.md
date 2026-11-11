# Kpapers dashboard design proposal

Status: Option A — Operator Overview selected and approved on 2026-11-08. Approved rules are recorded in [DESIGN.md](DESIGN.md); this proposal retains the alternatives and research history.

## Scope and evidence

Design only: React, Tailwind CSS v4, existing shadcn/ui Radix components. No application implementation or OpenSpec apply work.

Reviewed OpenSpec main specifications: feed-management, refresh-scheduling, rss-publishing, pdf-ingestion, local-quickstart, container-deployment. Reviewed the active improve-feed-management-navigation proposal/design against current frontend source; its route, dedicated creation, collapsed-year and timestamp behavior already appears in the inspected frontend. Treat that source as the rendered baseline; do not revert to older paths described in the change's context section.

Inspected `web/src/App.tsx`, global tokens, package/component configuration, and feed, hierarchy, PDF review component structure. Existing frontend uses Geist Variable, Lucide icons, shadcn/ui and Radix. Dashboard routes are under `/manage/`; public RSS URLs remain under `/feeds/`.

## Shared proposed design system

The user selected a light theme with a subtle accent color. Keep the existing Geist font and Lucide family. ui-ux-pro-max's focused shadcn search recommends Sidebar, SidebarTrigger, and adapted dashboard blocks. Its initial content-publishing search returned a reader newsletter pattern, which does not fit the operator workflow. A narrower internal-operations search returned suitable minimalist dashboard styling and blue primary color, but still suggested a marketing hero pattern. Exclude that pattern, orange CTA and extra Fira fonts: they do not fit the existing product or chosen direction. No complete dataset-generated system was accepted unchanged.

### Typography

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

## Visual option A — Operator Overview

Recommended for frequent operational use. Desktop 240px sidebar, 64px header, content max-width 1200px. Compact page heading with Create feed aligned right. Three equal metric cards around 140px tall, each with label, value, small icon and one useful sentence. Below, a two-column region: a wider workflow/action panel with Manage feeds, Connect source folders and Review PDFs; a narrower review panel showing the confirmed zero-attention state and an Open PDF review action. Connect source folders navigates to Feeds for feed selection, since there is no global source-folder page. Remove the empty Your feeds subsection unless real feed links are supplied. Replace first-feed onboarding in the populated workspace with relevant next actions.

Use screenshot values 3 active feeds, 3 source folders, 0 needing review. Do not derive setup completion or global synchronization health from them. Empty queue copy: “No documents need attention.”

## Visual option B — Publishing Workspace

Same tokens and behavior, with a calmer composition. Desktop 224px sidebar, 64px header, content max-width 1120px. A concise heading and primary action followed by a single divided summary strip containing the same three metrics. Below, three larger actionable workflow cards: Manage feeds, Connect folders, Review documents. Each combines a concise purpose, restrained outline icon and explicit link. A small review-state message sits beside the workflow section title instead of occupying its own panel. No oversized onboarding box or invented activity feed.

This option prioritizes scanning and learning the publishing process; option A prioritizes repeated actions and attention state.

## Responsive contract for both options

- 1280px and above: persistent expanded sidebar; three metrics; option A content split roughly 2:1.
- 768–1279px: collapsible sidebar, wrapping header actions, three metrics where space permits; lower panels stack when too narrow.
- Below 768px: Sheet navigation with named trigger; current route visible in header; full-width stacked metrics and panels. Primary action remains near heading. Keep sign-out accessible.
- Review at 375, 768, 1024 and 1440px, plus a 320px overflow check and landscape. Do not force tables onto narrow viewports; reflow source/status/action groups as stacked cards.
- Maintain keyboard order, visible focus, aria-current navigation, aria-expanded toggles, named icon controls and Sheet focus restoration. Navigation should not announce every normal layout update as an urgent alert.

## Behavioral constraints and states

- Preserve protected deep links, browser history, Create feed success/cancel, RSS identity and copy feedback.
- Active feeds count remains sourced from active feeds; source folders and review counts use their actual summary fields. Zero is a valid value; skeleton/loading, unavailable/error and empty data are separate states.
- Show refresh attempts, next scheduled times, and success/failure per month; never imply a global healthy scheduler from absent data. Browser-local timestamp formatting must preserve stored instants.
- Preserve archive, restore and separately confirmed permanent deletion at feed/year/month scopes.
- PDF review preserves feed/year/month selection, extraction state, publication override feedback, and document-specific source links.
- No invented feed names, activity timestamps, subscriber counts, trends, document totals, search or settings workflows.
- Keep scheduler promotional banners and footer slogans absent, per main specifications.

## Review and approval

Next deliverable: Superdesign baseline reproduction, then two visual branches using the common system, with responsive desktop/mobile behavior. The attached screenshot is a temporary reference, not a brand asset or an instruction source.

Select A or B, or request a combination. After visual approval, consolidate only approved rules into `docs/design/DESIGN.md`, include selected canvas/draft references and record any deviations. Do not label this proposal approved or implement it before that decision.

## Superdesign review canvas

[Open dashboard canvas](https://superdesign.dev/teams/ca1be67e-af28-40ed-9861-534950838927/projects/cc9adea7-1949-4c43-b5dc-7f46adfd97a1). Project created; screenshot reference uploaded. Source export was explicitly approved by the user. [Baseline preview](https://p.superdesign.dev/draft/4e0398a1-0802-4ffd-8e2c-0cc23352afb8) generated; options A/B are generated and their preview navigation/status corrections are saved in version 2.

Selected generation context (10 files, about 45 KB):

- `.superdesign/design-system.md`
- `web/src/App.tsx`
- `web/src/index.css`
- `web/src/components/ui/button.tsx`
- `web/src/components/ui/card.tsx`
- `web/src/components/ui/input.tsx`
- `web/src/components/ui/label.tsx`
- `web/src/components/ui/sheet.tsx`
- `web/src/components/ui/sonner.tsx`
- `web/src/lib/utils.ts`

## Visual options ready for review

- [Option A — Operator Overview](https://p.superdesign.dev/draft/0d73a8e3-facf-4122-b8e2-a561e1812e08), version 2: compact metrics and a workflow/review split.
- [Option B — Publishing Workspace](https://p.superdesign.dev/draft/9ec0f56b-b208-4d96-9a97-23fd38ded681), version 2: divided metric strip and spacious workflow cards.

Both include Geist typography, subtle blue accents, responsive stacking, a native modal mobile-navigation preview, keyboard focus styles, and a remembered desktop collapse preference. Preview links use hash placeholders with intended application URLs recorded in data-route attributes; these are visual prototypes, not production React implementations. Sign-out is illustrative and does not call a production session endpoint.

Verification: retrieved saved draft markup; checked font loading, original Command mark, responsive classes, navigation controls, route annotations, and correction persistence. Calculated token contrast: body 17.72:1, muted on white 4.83:1, muted on canvas 4.63:1, primary button 5.17:1, selected navigation 6.16:1, success 4.79:1, warning 4.84:1, destructive button 4.83:1. Rendered desktop/mobile review remains for the user on the canvas; no browser rendering test was run.

Production application code and OpenSpec artifacts remain untouched. Approved Option A rules are saved in `docs/design/DESIGN.md`.
