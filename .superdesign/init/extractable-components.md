# Extractable components
## Layout components
No independently exported layout components: Brand and Navigation are private functions embedded in App.tsx; DashboardShell wraps route children and auth actions. Extracting the entire shell would couple the preview to its nested page slot. Use full App.tsx context for this dashboard design.
## Button
Source: web/src/components/ui/button.tsx
Category: basic
Native button/slot; variant and size props.
## Card
Source: web/src/components/ui/card.tsx
Category: basic
CardHeader/CardTitle/CardDescription/CardContent composition.
## Badge
Source: web/src/components/ui/badge.tsx
Category: basic
Variant prop for status.
