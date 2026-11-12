# Design

## Context

See proposal.md for motivation and `docs/design/DESIGN.md` for the approved visual contract. The Vite React application already uses Tailwind v4, Radix shadcn components, Geist, manual React Router route parsing, and API-backed feed/year/month/document state. Navigation and hierarchy preferences already have versioned storage keys. The current feed component renders a list beside details and automatically selects a feed. Hierarchy forms remain visible inside expanded years. PDF badges currently substitute overrides for classification.

## Goals / Non-Goals

**Goals:** Implement all five approved views through composition of the existing stateful components. Retain behavior and real data while simplifying the information hierarchy. Cover changed interactions and actual responsive rendering.

**Non-Goals:** API or schema changes, new scheduling rules, new routes, redesigning sign-in, fabricated metrics, or replacing the existing UI library.

## Decisions

- Map approved colors and radii into the existing global semantic tokens and component variants. Use shared responsive control sizing and focus styles rather than per-page raw colors. Keep the Command mark black independently of the blue primary token.
- Keep existing route parsing and separate list/detail rendering based on selectedFeedId. Remove automatic feed selection on list entry. Use existing navigation callbacks for back links, history, and creation outcomes.
- Add only missing official shadcn Dialog, DropdownMenu, Tabs, Field, Alert, Empty, and InputGroup primitives if needed; reuse existing Button/Card/Badge/Sheet/AlertDialog/Skeleton. Radix supplies modal focus management, Escape, and trigger restoration. Prefer focused dialogs over expanding persistent forms; month metadata uses a named disclosure.
- Preserve all asynchronous API handlers and confirmation scopes. Dialog errors retain user input; successful operations close focused forms. Explicit labelled error summary links and aria-describedby connect form failures to fields.
- Keep PDF classification sourced from promotional_classification regardless of override. Preserve decision saving and extraction requests, displaying extraction requirements explicitly instead of implying that Include completes extraction.
- Use component tests for routing, state, failures, confirmations, dialog focus, and data fidelity. Extend deployed Playwright coverage for responsive sizes, overflow, navigation, and keyboard interactions. Existing unit tests that depend on the superseded master-detail/persistent-form layout must follow the new user paths.

## Risks / Trade-offs

- Modal composition can lose focus after an async mutation removes its trigger -> retain stable trigger identity and test restoration/cancel.
- Simplifying hierarchy can hide operations -> explicitly test every feed/year/month action and retain confirmation flows.
- Narrow viewports and long provider URLs can overflow -> min-width constraints, wrapping, stacked actions, and browser viewport assertions.
- Existing test selectors encode old layouts -> update them to the approved navigation and dialog workflows without dropping behavioral assertions.

## Migration Plan

Ship the frontend bundle with no backend migration. Existing deep links, RSS links, and preference keys remain valid. Rollback is a frontend bundle revert.
