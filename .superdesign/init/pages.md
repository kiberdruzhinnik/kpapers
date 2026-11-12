# Page dependency trees

## Overview + shared shell
Entry: web/src/App.tsx
- web/src/lib/api.ts
- web/src/components/ui/button.tsx
- web/src/components/ui/card.tsx
- web/src/components/ui/input.tsx
- web/src/components/ui/label.tsx
- web/src/components/ui/sheet.tsx
  - web/src/components/ui/button.tsx
- web/src/components/ui/sonner.tsx
- web/src/components/feeds/FeedManagement.tsx
  - web/src/lib/api.ts
  - web/src/components/ui/badge.tsx
  - web/src/components/ui/button.tsx
  - web/src/components/ui/card.tsx
  - web/src/components/ui/alert-dialog.tsx
    - web/src/components/ui/button.tsx
  - web/src/components/ui/input.tsx
  - web/src/components/ui/label.tsx
  - web/src/components/ui/textarea.tsx
  - web/src/components/feeds/HierarchyManagement.tsx
    - web/src/lib/api.ts
    - web/src/lib/datetime.ts
    - web/src/components/ui/badge.tsx
    - web/src/components/ui/button.tsx
    - web/src/components/ui/card.tsx
    - web/src/components/ui/alert-dialog.tsx
      - web/src/components/ui/button.tsx
    - web/src/components/ui/input.tsx
    - web/src/components/ui/label.tsx
- web/src/components/documents/PdfReview.tsx
  - web/src/lib/api.ts
  - web/src/components/ui/badge.tsx
  - web/src/components/ui/button.tsx
  - web/src/components/ui/card.tsx

## Feeds
Entry: web/src/components/feeds/FeedManagement.tsx
- web/src/lib/api.ts
- web/src/components/ui/badge.tsx
- web/src/components/ui/button.tsx
- web/src/components/ui/card.tsx
- web/src/components/ui/alert-dialog.tsx
  - web/src/components/ui/button.tsx
- web/src/components/ui/input.tsx
- web/src/components/ui/label.tsx
- web/src/components/ui/textarea.tsx
- web/src/components/feeds/HierarchyManagement.tsx
  - web/src/lib/api.ts
  - web/src/lib/datetime.ts
  - web/src/components/ui/badge.tsx
  - web/src/components/ui/button.tsx
  - web/src/components/ui/card.tsx
  - web/src/components/ui/alert-dialog.tsx
    - web/src/components/ui/button.tsx
  - web/src/components/ui/input.tsx
  - web/src/components/ui/label.tsx

## PDF review
Entry: web/src/components/documents/PdfReview.tsx
- web/src/lib/api.ts
- web/src/components/ui/badge.tsx
- web/src/components/ui/button.tsx
- web/src/components/ui/card.tsx

Global styling: web/src/index.css imported by web/src/main.tsx. App.css is starter CSS and is not imported. Tailwind v4 has no tailwind.config file.
