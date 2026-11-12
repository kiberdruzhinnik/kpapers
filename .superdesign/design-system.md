# Kpapers operator dashboard

## Product and routes
Kpapers is an operator workspace for author publishing feeds, year/month source folders, and PDF publication decisions. Existing pages: Overview (/manage/), Feeds (/manage/feeds), Create feed (/manage/feeds/new), feed detail, PDF review (/manage/pdf-review). Preserve these actions and terminology.

## Current baseline
React 19, Vite, shadcn/ui with Radix, Tailwind v4, Geist Variable sans-serif, Lucide outline icons. Expanded sidebar 248px, collapsed 72px; sticky header 64px. Overview has heading, Create feed CTA, three summary cards (Active feeds 3, Source folders 3, Needs review 0), then three-step Get started card. Sidebar currently shows a feed count without a feed list. Brand is a Command icon in a black rounded square and Kpapers wordmark, defined in App.tsx, with no separate logo asset.

## Chosen redesign direction
User selected: Light theme with a subtle accent color. Use blue #2563eb / blue-600 as restrained primary accent; blue-50 #eff6ff selected navigation and icon surfaces. Neutral zinc palette: canvas #fafafa, white surfaces, text #18181b, secondary #71717a, borders #e4e4e7. Green #15803d with #f0fdf4 only for confirmed clear review queue; amber #b45309 with #fffbeb for review attention. Baseline reproduction must retain current neutral primary; redesigned alternatives use the chosen blue accent.

## Typography and spacing
Geist sans-serif throughout, fallback Inter then system sans-serif. Headings 28px/34px semibold, section titles 16px/24px semibold, body 14px/21px, metadata 12px/18px, numeric metrics 30px semibold tabular. Use 4px spacing grid, 24-32px content gutters, 16-24px panel padding, 12-16px inter-card gaps. Border radius 8px buttons, 10-12px panels. Thin borders, minimal shadows; no gradients, photos, charts, decorative blobs or invented trend data.

## Components and behavior
Use shadcn/ui patterns: Sidebar, Breadcrumb, Button, Card, Badge, Separator, Table where supported. Maintain visible keyboard focus, semantic navigation and headings, accessible labels for icon buttons. Primary action Create feed; secondary View feeds and Open PDF review. Retain Command brand mark and Lucide icon family. Three concise stat cards should be about 140px high. Clear review queue can say No documents need attention; do not claim all documents published or all sources synchronized. Do not invent activity logs, author names, timestamps, growth rates, document totals or setup completion from the three available counts. Feed names absent in screenshot: hide empty Your feeds group or provide a Browse feeds shortcut rather than fictional names. Replace first-feed onboarding for nonempty workspace with useful workflow shortcuts or a compact publishing workflow guide. No unsupported global search/settings actions.

## Responsive and motion
Desktop sidebar 224-248px. On mobile use Sheet navigation and stacked cards/actions with no horizontal overflow. Hover/focus color transitions 120-180ms; respect reduced motion. Preview designs may show static screenshot values; implementation must preserve loading/error states and real data.

## Specification constraints
Design status: Option A approved by the user on 2026-11-08. Authoritative approved rules: docs/design/DESIGN.md. Research and alternatives: docs/design/PROPOSAL.md. Preserve /manage/ protected deep links, public /feeds/ RSS identity, sidebar preferences, feed/year/month hierarchy, archive/restore/delete confirmations, document-specific PDF links, and loading/error/zero-count distinctions. No scheduler promotional banner or footer slogans. Current frontend already has separate Create feed route and collapsible years; preserve them. Two requested alternatives: compact Operator Overview with metrics plus action/review split; spacious Publishing Workspace with divided metric strip and three workflow cards. Both retain the same Geist/blue/zinc design system.

## Approved selection
Option A — Operator Overview, draft 0d73a8e3-facf-4122-b8e2-a561e1812e08 version 2 is selected. Use 240px expanded / 72px collapsed sidebar, 64px header, content max-width 1200px, three compact metric cards and lower 2:1 workflow/review split. Use docs/design/DESIGN.md for final typography (28/36 page title, 14/20 body), spacing and responsive rules. Option B remains an unselected comparison. Production code changes are not authorized by design approval.
