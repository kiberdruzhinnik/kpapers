# Shared layout
App.tsx contains Brand, Navigation, DashboardShell and SignInPanel. Expanded desktop sidebar 248px, collapsed 72px; mobile Sheet, sticky 64px header, content max-width 1440px.

### `web/src/App.tsx`
```tsx
import { useCallback, useEffect, useMemo, useState } from "react"
import { BookOpenText, ChevronDown, Command, Files, LayoutDashboard, LogOut, Menu, PanelLeftClose, PanelLeftOpen, Plus, Rss, ShieldCheck } from "lucide-react"
import { BrowserRouter, Link, useLocation, useNavigate } from "react-router-dom"
import { ApiError, api, type Feed, type OverviewSummary } from "@/lib/api"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from "@/components/ui/sheet"
import { Toaster } from "@/components/ui/sonner"
import { CreateFeed, FeedManagement } from "@/components/feeds/FeedManagement"
import { PdfReview } from "@/components/documents/PdfReview"
import { toast } from "sonner"

type SessionState = "checking" | "signed-out" | "signed-in"
export type Section = "Overview" | "Feeds" | "PDF review"
const SIDEBAR_STORAGE_KEY = "kpapers.sidebar.collapsed.v1"

const navItems: { label: Section; description: string; icon: typeof LayoutDashboard }[] = [
  { label: "Overview", description: "Workspace summary", icon: LayoutDashboard },
  { label: "Feeds", description: "Authors and sources", icon: Files },
  { label: "PDF review", description: "Publication decisions", icon: BookOpenText },
]

function Brand({ compact = false }: { compact?: boolean }) {
  return (
    <div className="flex items-center gap-3">
      <div className="flex size-9 items-center justify-center rounded-xl bg-primary text-primary-foreground shadow-sm">
        <Command aria-hidden="true" className="size-4" strokeWidth={2.4} />
      </div>
      {!compact && <div className="leading-tight">
        <div className="font-semibold tracking-tight">Kpapers</div>
        <div className="mt-1 text-[11px] text-muted-foreground">OPERATOR DESK</div>
      </div>}
    </div>
  )
}

type ReviewSelection = { feedId: string; year?: number; monthId?: string }
type DashboardRoute = { creating?: boolean; section: Section; feedId?: string; reviewSelection?: ReviewSelection } | null

function routeFromPath(pathname: string): DashboardRoute {
  if (["/", "/manage", "/manage/"].includes(pathname)) return { section: "Overview" }
  if (pathname === "/manage/feeds/new") return { section: "Feeds", creating: true }
  if (pathname === "/manage/feeds") return { section: "Feeds" }
  if (pathname === "/manage/pdf-review") return { section: "PDF review" }

  const feedMatch = pathname.match(/^\/manage\/feeds\/([^/]+)$/)
  if (feedMatch) {
    try { return { section: "Feeds", feedId: decodeURIComponent(feedMatch[1]) } } catch { return null }
  }

  const reviewMatch = pathname.match(/^\/manage\/pdf-review\/([^/]+)(?:\/([^/]+)(?:\/([^/]+))?)?$/)
  if (reviewMatch) {
    const year = reviewMatch[2] === undefined ? undefined : Number(reviewMatch[2])
    if (year !== undefined && (!Number.isInteger(year) || year < 1 || year > 9999)) return null
    try {
      return {
        section: "PDF review",
        reviewSelection: {
          feedId: decodeURIComponent(reviewMatch[1]),
          ...(year === undefined ? {} : { year }),
          ...(reviewMatch[3] === undefined ? {} : { monthId: decodeURIComponent(reviewMatch[3]) }),
        },
      }
    } catch { return null }
  }

  return null
}

function sectionPath(section: Section) {
  return section === "Overview" ? "/manage/" : section === "Feeds" ? "/manage/feeds" : "/manage/pdf-review"
}

function feedPath(feedId?: string | null) {
  return feedId ? `/manage/feeds/${encodeURIComponent(feedId)}` : sectionPath("Feeds")
}

function storedSidebarState() {
  try { return window.localStorage.getItem(SIDEBAR_STORAGE_KEY) === "collapsed" } catch { return false }
}

function reviewPath(selection: ReviewSelection) {
  const feedPath = `/manage/pdf-review/${encodeURIComponent(selection.feedId)}`
  if (selection.year === undefined) return feedPath
  if (selection.monthId === undefined) return `${feedPath}/${selection.year}`
  return `${feedPath}/${selection.year}/${encodeURIComponent(selection.monthId)}`
}

export function SignInPanel({ onSubmit, error, pending }: {
  onSubmit: (username: string, password: string) => Promise<void>
  error: string | null
  pending: boolean
}) {
  async function submit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const data = new FormData(event.currentTarget)
    await onSubmit(String(data.get("username") ?? ""), String(data.get("password") ?? ""))
  }

  return (
    <main className="flex min-h-svh items-center justify-center bg-muted/40 px-4 py-10">
      <div className="w-full max-w-[420px]">
        <div className="mb-8 flex justify-center"><Brand /></div>
        <Card className="border-border/80 shadow-lg shadow-black/[0.03]">
          <CardHeader className="gap-2 px-7 pt-7">
            <h1 className="text-xl font-semibold tracking-tight">Welcome back</h1>
            <CardDescription>Sign in to manage author feeds and source folders.</CardDescription>
          </CardHeader>
          <CardContent className="px-7 pb-7">
            <form className="space-y-5" onSubmit={submit}>
              <div className="space-y-2">
                <Label htmlFor="username">Username</Label>
                <Input id="username" name="username" autoComplete="username" autoCapitalize="none" required autoFocus />
              </div>
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <Label htmlFor="password">Password</Label>
                  <span className="text-xs text-muted-foreground">Operator access</span>
                </div>
                <Input id="password" name="password" type="password" autoComplete="current-password" required />
              </div>
              {error && <p role="alert" className="rounded-lg border border-destructive/20 bg-destructive/5 px-3 py-2.5 text-sm text-destructive">{error}</p>}
              <Button className="w-full" size="lg" type="submit" disabled={pending}>
                {pending ? "Signing in…" : "Sign in"}
              </Button>
            </form>
          </CardContent>
        </Card>
        <p className="mt-6 flex items-center justify-center gap-2 text-xs text-muted-foreground">
          <ShieldCheck aria-hidden="true" className="size-3.5" /> Your operator session is protected.
        </p>
      </div>
    </main>
  )
}

function Navigation({ section, onNavigate, onSelect, collapsed = false }: {
  section: Section
  onNavigate: (section: Section) => void
  onSelect?: () => void
  collapsed?: boolean
}) {
  return (
    <nav aria-label="Primary navigation" className="space-y-1">
      <p className={`px-3 pb-2 pt-1 text-[11px] font-semibold uppercase tracking-[0.12em] text-muted-foreground ${collapsed ? "sr-only" : ""}`}>Workspace</p>
      {navItems.map(({ label, description, icon: Icon }) => (
        <button
          key={label}
          aria-current={section === label ? "page" : undefined}
          aria-label={`${label} — ${description}`}
          className={`flex w-full items-center ${collapsed ? "justify-center px-2" : "gap-3 px-3 text-left"} rounded-lg py-2.5 text-sm transition-colors ${section === label ? "bg-primary/8 font-medium text-foreground" : "text-muted-foreground hover:bg-muted hover:text-foreground"}`}
          title={collapsed ? label : undefined}
          onClick={() => { onNavigate(label); onSelect?.() }}
          type="button"
        >
          <Icon aria-hidden="true" className={`size-4 ${section === label ? "text-primary" : ""}`} />
          <span className={collapsed ? "sr-only" : ""}>{label}</span>
          {!collapsed && label === "Feeds" && <ChevronDown aria-hidden="true" className="ml-auto size-3.5 -rotate-90 opacity-40" />}
        </button>
      ))}
    </nav>
  )
}

export function DashboardShell({ children, onLogout, feedCount = 0, section, onSectionChange }: {
  children: React.ReactNode
  onLogout: () => void
  feedCount?: number
  section: Section
  onSectionChange: (section: Section) => void
}) {
  const [menuOpen, setMenuOpen] = useState(false)
  const [sidebarCollapsed, setSidebarCollapsed] = useState(storedSidebarState)

  useEffect(() => {
    try { window.localStorage.setItem(SIDEBAR_STORAGE_KEY, sidebarCollapsed ? "collapsed" : "expanded") } catch { /* Storage may be unavailable in restricted browsers. */ }
  }, [sidebarCollapsed])

  return (
    <div className="min-h-svh bg-muted/30 text-foreground">
      <div className="flex min-h-svh">
        <aside className={`hidden shrink-0 flex-col border-r bg-background py-5 transition-[width] md:flex ${sidebarCollapsed ? "w-[72px] px-2" : "w-[248px] px-4"}`}>
          <div className={`flex items-start pb-8 ${sidebarCollapsed ? "flex-col items-center gap-3" : "justify-between gap-2 px-2"}`}>
            <Link aria-label="Kpapers overview" to="/manage/"><Brand compact={sidebarCollapsed} /></Link>
            <Button aria-expanded={!sidebarCollapsed} aria-label={sidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"} onClick={() => setSidebarCollapsed((collapsed) => !collapsed)} size="icon" title={sidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"} variant="ghost">
              {sidebarCollapsed ? <PanelLeftOpen aria-hidden="true" /> : <PanelLeftClose aria-hidden="true" />}
            </Button>
          </div>
          <Navigation collapsed={sidebarCollapsed} section={section} onNavigate={onSectionChange} />
          <div className={`mt-7 ${sidebarCollapsed ? "flex justify-center" : "px-3"}`}>
            <div className={`pb-2 ${sidebarCollapsed ? "sr-only" : "flex items-center justify-between"}`}>
              <span className="text-[11px] font-semibold uppercase tracking-[0.12em] text-muted-foreground">Your feeds</span>
              {!sidebarCollapsed && <span className="rounded-md bg-muted px-1.5 py-0.5 text-[11px] tabular-nums text-muted-foreground">{feedCount}</span>}
            </div>
            {sidebarCollapsed
              ? <span aria-label={`${feedCount} active feeds`} className="rounded-md bg-muted px-1.5 py-0.5 text-[11px] tabular-nums text-muted-foreground">{feedCount}</span>
              : feedCount === 0 && <p className="text-xs leading-relaxed text-muted-foreground">Your author feeds will appear here.</p>}
          </div>
        </aside>

        <div className="flex min-w-0 flex-1 flex-col">
          <header className="sticky top-0 z-20 flex h-16 items-center justify-between border-b bg-background/95 px-4 backdrop-blur supports-backdrop-filter:bg-background/80 sm:px-6 lg:px-8">
            <div className="flex items-center gap-3">
              <Sheet open={menuOpen} onOpenChange={setMenuOpen}>
                <SheetTrigger asChild>
                  <Button aria-label="Open navigation" className="md:hidden" size="icon" variant="ghost"><Menu aria-hidden="true" /></Button>
                </SheetTrigger>
                <SheetContent className="w-[280px] max-w-[85vw] p-4" side="left">
                  <SheetHeader className="px-2 pb-5"><SheetTitle><Link aria-label="Kpapers overview" to="/manage/" onClick={() => setMenuOpen(false)}><Brand /></Link></SheetTitle></SheetHeader>
                  <Navigation section={section} onNavigate={onSectionChange} onSelect={() => setMenuOpen(false)} />
                </SheetContent>
              </Sheet>
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <span className="hidden sm:inline">Workspace</span><span className="hidden text-border sm:inline">/</span>
                <span className="font-medium text-foreground">{section}</span>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <div className="hidden items-center gap-2 rounded-lg border bg-background px-2.5 py-1.5 sm:flex">
                <div className="flex size-6 items-center justify-center rounded-full bg-primary/10 text-[10px] font-semibold text-primary">OP</div>
                <span className="text-xs font-medium">Operator</span>
              </div>
              <Button onClick={onLogout} size="sm" variant="ghost"><LogOut aria-hidden="true" /> <span className="hidden sm:inline">Sign out</span></Button>
            </div>
          </header>
          <main aria-live="polite" className="mx-auto w-full max-w-[1440px] flex-1 px-4 py-7 sm:px-6 sm:py-8 lg:px-10">
            {children}
          </main>
        </div>
      </div>
    </div>
  )
}

function Overview({ feedCount, onNavigate, onCreate, summary, summaryLoading, summaryError }: {
  feedCount: number
  onNavigate: (section: Section) => void
  onCreate: () => void
  summary: OverviewSummary | null
  summaryLoading: boolean
  summaryError: string | null
}) {
  const metric = (value: number | undefined) => summaryLoading ? "Loading…" : summaryError || value === undefined ? "Unavailable" : value.toLocaleString()

  return (
    <div className="space-y-7">
      <div className="flex flex-wrap items-end justify-between gap-4">
        <div><p className="mb-2 text-sm text-muted-foreground">Your publishing workspace</p><h1 className="text-2xl font-semibold tracking-tight sm:text-[28px]">Overview</h1><p className="mt-2 max-w-xl text-sm text-muted-foreground">Manage author feeds, keep source folders organized, and review documents before they reach readers.</p></div>
        <Button onClick={onCreate}><Plus aria-hidden="true" /> Create feed</Button>
      </div>

      <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
        <Card>
          <CardHeader className="flex-row items-start justify-between space-y-0 pb-2">
            <div><CardDescription>Active feeds</CardDescription><CardTitle className="mt-2 text-3xl tabular-nums">{feedCount}</CardTitle></div>
            <div className="flex size-9 items-center justify-center rounded-lg bg-primary/8 text-primary"><Rss aria-hidden="true" className="size-4" /></div>
          </CardHeader>
          <CardContent><p className="text-xs text-muted-foreground">Author channels available to subscribers</p></CardContent>
        </Card>
        <Card>
          <CardHeader className="flex-row items-start justify-between space-y-0 pb-2">
            <div><CardDescription>Source folders</CardDescription><CardTitle className="mt-2 text-3xl tabular-nums" data-testid="source-folder-count">{metric(summary?.source_folder_count)}</CardTitle></div>
            <div className="flex size-9 items-center justify-center rounded-lg bg-blue-500/10 text-blue-600"><Files aria-hidden="true" className="size-4" /></div>
          </CardHeader>
          <CardContent><p className="text-xs text-muted-foreground">Configured source folders for active feeds</p>{summaryError && <p className="mt-1 text-xs text-destructive">Count unavailable</p>}</CardContent>
        </Card>
        <Card className="sm:col-span-2 xl:col-span-1">
          <CardHeader className="flex-row items-start justify-between space-y-0 pb-2">
            <div><CardDescription>Needs review</CardDescription><CardTitle className="mt-2 text-3xl tabular-nums" data-testid="needs-review-count">{metric(summary?.needs_review_count)}</CardTitle></div>
            <div className="flex size-9 items-center justify-center rounded-lg bg-amber-500/10 text-amber-700"><BookOpenText aria-hidden="true" className="size-4" /></div>
          </CardHeader>
          <CardContent><p className="text-xs text-muted-foreground">PDFs with incomplete or failed classification processing</p>{summaryError && <p className="mt-1 text-xs text-destructive">Count unavailable</p>}</CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader className="flex-row flex-wrap items-start justify-between gap-3">
          <div><CardTitle>Get started</CardTitle><CardDescription className="mt-1.5">Set up your first author feed in a few steps.</CardDescription></div>
          <Button onClick={() => onNavigate("Feeds")} variant="outline">View feeds <ChevronDown aria-hidden="true" className="-rotate-90" /></Button>
        </CardHeader>
        <CardContent className="grid gap-4 border-t pt-5 md:grid-cols-3">
          {[
            { icon: Plus, title: "Create a feed", body: "Add an author name and description for the public channel." },
            { icon: Files, title: "Connect folders", body: "Add years and link only months that have a source folder." },
            { icon: BookOpenText, title: "Review documents", body: "Check discovered PDFs and choose what readers can see." },
          ].map(({ icon: Icon, title, body }, index) => <div className="flex gap-3" key={title}>
            <div className="flex size-8 shrink-0 items-center justify-center rounded-lg border bg-background text-muted-foreground"><Icon aria-hidden="true" className="size-4" /></div>
            <div className="min-w-0"><p className="text-sm font-medium"><span className="mr-2 text-xs text-muted-foreground">0{index + 1}</span>{title}</p><p className="mt-1.5 text-xs leading-relaxed text-muted-foreground">{body}</p></div>
          </div>)}
        </CardContent>
      </Card>
    </div>
  )
}

function AppContent() {
  const navigate = useNavigate()
  const location = useLocation()
  const [session, setSession] = useState<SessionState>("checking")
  const [authError, setAuthError] = useState<string | null>(null)
  const [loginPending, setLoginPending] = useState(false)
  const [feeds, setFeeds] = useState<Feed[]>([])
  const [summary, setSummary] = useState<OverviewSummary | null>(null)
  const [summaryLoading, setSummaryLoading] = useState(true)
  const [summaryError, setSummaryError] = useState<string | null>(null)
  const route = useMemo(() => routeFromPath(location.pathname), [location.pathname])
  const section = route?.section ?? "Overview"
  const reviewSelection = route?.reviewSelection ?? null
  const requestedFeedId = route?.feedId ?? reviewSelection?.feedId
  const updateReviewSelection = useCallback((selection: ReviewSelection | null) => {
    navigate(selection ? reviewPath(selection) : "/manage/pdf-review")
  }, [navigate])

  const restoreSession = useCallback(async () => {
    try {
      await api.session()
      const loadedFeeds = await api.feeds()
      setFeeds(loadedFeeds)
      setSession("signed-in")
      setAuthError(null)
      return true
    } catch (error) {
      setFeeds([])
      if (error instanceof ApiError && error.status === 401) {
        setSession("signed-out")
        return false
      }
      setSession("signed-out")
      setAuthError("Kpapers is temporarily unavailable. Please try again.")
      return false
    }
  }, [])

  useEffect(() => {
    if (location.pathname === "/" || location.pathname === "/manage") navigate("/manage/", { replace: true })
  }, [location.pathname, navigate])

  useEffect(() => {
    if (!route) {
      toast.error("That dashboard page could not be found.")
      navigate("/manage/", { replace: true })
    }
  }, [navigate, route])

  useEffect(() => {
    if (session !== "signed-in" || !requestedFeedId) return
    if (feeds.some((feed) => feed.id === requestedFeedId)) return
    toast.error("That feed is no longer available.")
    navigate(section === "PDF review" ? "/manage/pdf-review" : "/manage/feeds", { replace: true })
  }, [feeds, navigate, requestedFeedId, section, session])

  useEffect(() => {
    if (session !== "signed-in" || section !== "Overview") return
    let mounted = true
    setSummaryLoading(true)
    setSummaryError(null)
    api.overview().then((result) => {
      if (mounted) setSummary(result)
    }).catch(() => {
      if (mounted) setSummaryError("Overview counts could not be loaded.")
    }).finally(() => {
      if (mounted) setSummaryLoading(false)
    })
    return () => { mounted = false }
  }, [section, session])

  useEffect(() => {
    let mounted = true
    async function restore() {
      try {
        await api.session()
        const loadedFeeds = await api.feeds()
        if (!mounted) return
        setFeeds(loadedFeeds)
        setSession("signed-in")
      } catch (error) {
        if (!mounted) return
        setFeeds([])
        setSession("signed-out")
        if (!(error instanceof ApiError && error.status === 401)) {
          setAuthError("Kpapers is temporarily unavailable. Please try again.")
        }
      }
    }
    void restore()
    return () => { mounted = false }
  }, [])

  async function signIn(username: string, password: string) {
    setLoginPending(true)
    setAuthError(null)
    try {
      await api.login(username, password)
      if (!await restoreSession()) throw new Error("Kpapers could not restore your session. Please try again.")
      setSession("signed-in")
      toast.success("Signed in", { description: "Your publishing workspace is ready." })
    } catch (error) {
      if (error instanceof ApiError && error.status === 401) setAuthError("The username or password is incorrect.")
      else setAuthError(error instanceof Error ? error.message : "Sign in failed. Please try again.")
    } finally {
      setLoginPending(false)
    }
  }

  async function signOut() {
    try {
      await api.logout()
      setFeeds([])
      setSession("signed-out")
      navigate("/manage/", { replace: true })
      toast.success("Signed out")
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Could not sign out. Please try again.")
    }
  }

  if (session === "checking") return <div aria-label="Loading workspace" className="flex min-h-svh items-center justify-center bg-muted/30"><div className="flex items-center gap-3 text-sm text-muted-foreground"><span className="size-4 animate-spin rounded-full border-2 border-muted-foreground/30 border-t-primary" />Loading your workspace…</div></div>

  if (!route) return null

  return (
    <>
      <Toaster position="top-right" />
      {session === "signed-out"
        ? <SignInPanel error={authError} onSubmit={signIn} pending={loginPending} />
        : <DashboardShell feedCount={feeds.filter((feed) => feed.active).length} onLogout={() => void signOut()} onSectionChange={(nextSection) => navigate(sectionPath(nextSection))} section={section}>
          {section === "Overview"
            ? <Overview onCreate={() => navigate("/manage/feeds/new")} feedCount={feeds.filter((feed) => feed.active).length} onNavigate={(nextSection) => navigate(sectionPath(nextSection))} summary={summary} summaryLoading={summaryLoading} summaryError={summaryError} />
            : section === "Feeds"
              ? route.creating
                ? <CreateFeed onCancel={() => navigate("/manage/feeds")} onCreated={(feed) => { setFeeds((current) => [...current, feed]); navigate(feedPath(feed.id)) }} />
                : <FeedManagement onCreate={() => navigate("/manage/feeds/new")} feeds={feeds} onFeedsChange={setFeeds} onReviewMonth={(month) => navigate(reviewPath({ feedId: month.feed_id, year: month.year, monthId: month.id }))} onSelectFeed={(feedId) => navigate(feedPath(feedId))} selectedFeedId={route.feedId ?? null} />
              : <PdfReview feeds={feeds} initialSelection={reviewSelection} key={location.pathname} onBack={() => navigate(feedPath(reviewSelection?.feedId))} onInvalidSelection={() => navigate("/manage/pdf-review", { replace: true })} onSelectionChange={updateReviewSelection} />}
        </DashboardShell>}
    </>
  )
}

export default function App() {
  return <BrowserRouter><AppContent /></BrowserRouter>
}

```
