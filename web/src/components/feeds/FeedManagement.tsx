import { useEffect, useMemo, useState, type FormEvent } from "react"
import { Archive, Check, ChevronRight, Copy, FileText, Plus, RefreshCw, Rss, Trash2, Undo2 } from "lucide-react"
import { toast } from "sonner"
import { api, type Feed, type Month } from "@/lib/api"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
import { HierarchyManagement } from "@/components/feeds/HierarchyManagement"

type FeedFormValues = { display_name: string; description: string; refresh_hours: string }
type ConfirmAction = { kind: "archive" | "delete"; feed: Feed }

function secondsFromHours(raw: string): number | null {
  const hours = Number(raw)
  return Number.isFinite(hours) && hours > 0 ? Math.round(hours * 3600) : null
}

function hoursFromSeconds(seconds: number | null): string {
  return seconds ? String(seconds / 3600) : ""
}

function FeedForm({ initial, submitLabel, onSubmit, onCancel }: {
  initial?: Feed
  submitLabel: string
  onSubmit: (values: { display_name: string; description: string; refresh_interval_seconds: number | null }) => Promise<void>
  onCancel?: () => void
}) {
  const [values, setValues] = useState<FeedFormValues>(() => ({
    display_name: initial?.display_name ?? "",
    description: initial?.description ?? "",
    refresh_hours: hoursFromSeconds(initial?.refresh_interval_seconds ?? null),
  }))
  const [pending, setPending] = useState(false)
  const [error, setError] = useState<string | null>(null)

  function update(name: keyof FeedFormValues, value: string) {
    setValues((current) => ({ ...current, [name]: value }))
    setError(null)
  }

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const name = values.display_name.trim()
    if (!name) {
      setError("Enter an author name.")
      return
    }
    if (values.refresh_hours && (!Number.isFinite(Number(values.refresh_hours)) || Number(values.refresh_hours) <= 0)) {
      setError("Refresh interval must be greater than zero hours.")
      return
    }
    setPending(true)
    try {
      await onSubmit({
        display_name: name,
        description: values.description.trim(),
        refresh_interval_seconds: secondsFromHours(values.refresh_hours),
      })
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Could not save feed. Please try again.")
    } finally {
      setPending(false)
    }
  }

  const id = initial ? `edit-feed-${initial.id}` : "create-feed"
  return (
    <form className="space-y-4" onSubmit={(event) => void submit(event)}>
      <div className="space-y-2">
        <Label htmlFor={`${id}-name`}>Author name</Label>
        <Input autoComplete="off" id={`${id}-name`} name="display_name" onChange={(event) => update("display_name", event.target.value)} placeholder="e.g. Ada Lovelace" required value={values.display_name} />
      </div>
      <div className="space-y-2">
        <Label htmlFor={`${id}-description`}>Description <span className="font-normal text-muted-foreground">(optional)</span></Label>
        <Textarea id={`${id}-description`} name="description" onChange={(event) => update("description", event.target.value)} placeholder="A short introduction for readers" rows={3} value={values.description} />
      </div>
      <div className="space-y-2">
        <Label htmlFor={`${id}-refresh`}>Refresh interval <span className="font-normal text-muted-foreground">(optional)</span></Label>
        <div className="relative">
          <Input className="pr-14" id={`${id}-refresh`} max="87600" name="refresh_hours" onChange={(event) => update("refresh_hours", event.target.value)} placeholder="Use defaults" step="0.1" type="number" value={values.refresh_hours} />
          <span className="pointer-events-none absolute inset-y-0 right-3 flex items-center text-xs text-muted-foreground">hours</span>
        </div>
        <p className="text-xs text-muted-foreground">Leave blank to use the current and older month defaults.</p>
      </div>
      {error && <p role="alert" className="rounded-md bg-destructive/5 px-3 py-2 text-sm text-destructive">{error}</p>}
      <div className="flex flex-wrap gap-2 pt-1">
        {onCancel && <Button onClick={onCancel} type="button" variant="outline">Cancel</Button>}
        <Button disabled={pending} type="submit">{pending ? "Saving…" : submitLabel}</Button>
      </div>
    </form>
  )
}

function FeedDetails({ feed, onSaved, onRequestConfirm, onReviewMonth }: {
  feed: Feed
  onSaved: () => Promise<void>
  onRequestConfirm: (action: ConfirmAction) => void
  onReviewMonth: (month: Month) => void
}) {
  const [editing, setEditing] = useState(false)
  const [refreshing, setRefreshing] = useState(false)
  const [copied, setCopied] = useState(false)
  const rssUrl = `${window.location.origin}/feeds/${feed.id}`

  async function save(values: { display_name: string; description: string; refresh_interval_seconds: number | null }) {
    try {
      await api.updateFeed(feed.id, values)
      await onSaved()
      setEditing(false)
      toast.success("Feed details saved", { description: "The RSS address stays the same." })
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Could not save feed.")
      throw error
    }
  }

  async function refresh() {
    setRefreshing(true)
    try {
      await api.refreshFeed(feed.id)
      toast.success("Feed refresh queued", { description: "Configured source folders will be refetched." })
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Could not queue refresh.")
    } finally {
      setRefreshing(false)
    }
  }

  async function copyUrl() {
    try {
      if (navigator.clipboard?.writeText) {
        try {
          await navigator.clipboard.writeText(rssUrl)
        } catch {
          if (!copyTextWithDocument(rssUrl)) throw new Error("Clipboard write failed")
        }
      } else if (!copyTextWithDocument(rssUrl)) {
        throw new Error("Clipboard is unavailable")
      }
      setCopied(true)
      toast.success("RSS address copied")
      window.setTimeout(() => setCopied(false), 1800)
    } catch {
      toast.error("Could not copy the RSS address. Select and copy it instead.")
    }
  }

  return (
    <div className="space-y-5">
      <Card>
        <CardHeader className="flex flex-row flex-wrap items-start justify-between gap-4">
          <div className="flex min-w-0 gap-3">
            <div className="flex size-11 shrink-0 items-center justify-center rounded-xl bg-primary/8 text-primary"><Rss aria-hidden="true" className="size-5" /></div>
            <div className="min-w-0"><div className="flex flex-wrap items-center gap-2"><h2 className="text-xl font-semibold tracking-tight">{feed.display_name}</h2><Badge variant="secondary">{feed.active ? "Active" : "Archived"}</Badge></div><CardDescription className="mt-1.5 max-w-2xl">{feed.description || "No description added yet."}</CardDescription></div>
          </div>
          {feed.active && <div className="flex flex-wrap gap-2">
            <Button disabled={refreshing} onClick={() => void refresh()} variant="outline"><RefreshCw aria-hidden="true" className={refreshing ? "animate-spin" : ""} /> Refresh feed</Button>
            <Button onClick={() => setEditing((current) => !current)} variant="outline"><FileText aria-hidden="true" /> {editing ? "Close editor" : "Edit details"}</Button>
            <Button aria-label={`More actions for ${feed.display_name}`} onClick={() => onRequestConfirm({ kind: "archive", feed })} size="icon" variant="ghost"><Archive aria-hidden="true" /></Button>
          </div>}
          {!feed.active && <div className="flex flex-wrap gap-2">
            <Button onClick={async () => { try { await api.restoreFeed(feed.id); await onSaved(); toast.success("Feed restored") } catch (error) { toast.error(error instanceof Error ? error.message : "Could not restore feed.") } }} variant="outline"><Undo2 aria-hidden="true" /> Restore</Button>
            <Button onClick={() => onRequestConfirm({ kind: "delete", feed })} variant="destructive"><Trash2 aria-hidden="true" /> Delete permanently</Button>
          </div>}
        </CardHeader>
        <CardContent className="border-t pt-4">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="min-w-0"><p className="text-xs font-medium text-muted-foreground">Stable RSS address</p><code className="mt-1 block break-all rounded bg-muted px-2 py-1 text-xs text-foreground">{rssUrl}</code></div>
            <Button aria-label="Copy RSS address" onClick={() => void copyUrl()} size="sm" variant="outline">{copied ? <Check aria-hidden="true" /> : <Copy aria-hidden="true" />} {copied ? "Copied" : "Copy link"}</Button>
          </div>
        </CardContent>
      </Card>

      {editing && <Card>
        <CardHeader><CardTitle>Edit feed details</CardTitle><CardDescription>Changes to the display name do not change the RSS address.</CardDescription></CardHeader>
        <CardContent><FeedForm initial={feed} onCancel={() => setEditing(false)} onSubmit={save} submitLabel="Save changes" /></CardContent>
      </Card>}

      <HierarchyManagement active={feed.active} feedId={feed.id} onReviewMonth={onReviewMonth} />

      <div className="flex flex-wrap gap-x-5 gap-y-2 text-xs text-muted-foreground">
        <span>Feed ID <code className="ml-1 break-all text-[11px] text-foreground/70">{feed.id}</code></span>
        <span>Refresh interval {feed.refresh_interval_seconds ? `${(feed.refresh_interval_seconds / 3600).toLocaleString()} hours` : "Default schedule"}</span>
      </div>
    </div>
  )
}

function copyTextWithDocument(value: string) {
  const input = document.createElement("textarea")
  input.value = value
  input.setAttribute("readonly", "")
  input.style.position = "fixed"
  input.style.opacity = "0"
  document.body.appendChild(input)
  input.select()
  input.setSelectionRange(0, input.value.length)
  try {
    return document.execCommand("copy")
  } catch {
    return false
  } finally {
    input.remove()
  }
}

export function FeedManagement({ feeds, onFeedsChange, onReviewMonth, selectedFeedId, onSelectFeed, onCreate }: {
  onCreate: () => void
  feeds: Feed[]
  onFeedsChange: (feeds: Feed[]) => void
  onReviewMonth: (month: Month) => void
  selectedFeedId?: string | null
  onSelectFeed?: (feedId: string | null) => void
}) {
  const activeFeeds = useMemo(() => feeds.filter((feed) => feed.active), [feeds])
  const archivedFeeds = useMemo(() => feeds.filter((feed) => !feed.active), [feeds])
  const [localSelectedId, setLocalSelectedId] = useState<string | null>(() => activeFeeds[0]?.id ?? null)
  const selectedId = selectedFeedId === undefined ? localSelectedId : selectedFeedId
  const [confirmAction, setConfirmAction] = useState<ConfirmAction | null>(null)
  const selectedFeed = feeds.find((feed) => feed.id === selectedId) ?? null

  function selectFeed(id: string | null) {
    setLocalSelectedId(id)
    onSelectFeed?.(id)
  }

  useEffect(() => {
    if (selectedFeedId !== undefined) setLocalSelectedId(selectedFeedId)
  }, [selectedFeedId])

  async function reloadFeeds(selectId?: string) {
    const loaded = await api.feeds()
    onFeedsChange(loaded)
    if (selectId) selectFeed(selectId)
    else if (selectedId && !loaded.some((feed) => feed.id === selectedId && feed.active)) selectFeed(loaded.find((feed) => feed.active)?.id ?? null)
  }

  async function confirm() {
    if (!confirmAction) return
    const { kind, feed } = confirmAction
    try {
      if (kind === "archive") {
        await api.archiveFeed(feed.id)
        toast.success("Feed archived", { description: "You can restore it at any time." })
      } else {
        await api.deleteFeed(feed.id)
        toast.success("Feed permanently deleted")
      }
      setConfirmAction(null)
      await reloadFeeds()
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Could not update feed.")
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-end justify-between gap-4">
        <div><p className="mb-2 text-sm text-muted-foreground">Publishing workspace</p><h1 className="text-2xl font-semibold tracking-tight sm:text-[28px]">Feeds</h1><p className="mt-2 max-w-xl text-sm text-muted-foreground">Create author channels, organize source folders, and keep public RSS addresses stable.</p></div>
        <Button onClick={onCreate}><Plus aria-hidden="true" />Create feed</Button>
        <Badge className="h-7 rounded-full px-3" variant="secondary">{activeFeeds.length} active · {archivedFeeds.length} archived</Badge>
      </div>
      <div className="grid min-w-0 gap-5 xl:grid-cols-[minmax(260px,320px)_minmax(0,1fr)]">
        <aside className="space-y-5">
          <Card>
            <CardHeader className="pb-3"><CardTitle className="text-base">Your feeds</CardTitle><CardDescription>Select a feed to manage its details.</CardDescription></CardHeader>
            <CardContent className="space-y-1 px-2 pb-3">
              {activeFeeds.length === 0 && <p className="px-2 py-3 text-sm text-muted-foreground">No active feeds yet.</p>}
              {activeFeeds.map((feed) => <button aria-current={selectedId === feed.id ? "page" : undefined} className={`flex w-full items-center gap-3 rounded-lg px-2.5 py-2.5 text-left transition-colors ${selectedId === feed.id ? "bg-primary/8 text-foreground" : "text-muted-foreground hover:bg-muted hover:text-foreground"}`} key={feed.id} onClick={() => selectFeed(feed.id)} type="button">
                <span className="flex size-8 shrink-0 items-center justify-center rounded-lg border bg-background"><Rss aria-hidden="true" className="size-3.5" /></span>
                <span className="min-w-0 flex-1"><span className="block truncate text-sm font-medium">{feed.display_name}</span><span className="mt-0.5 block truncate text-xs text-muted-foreground">{feed.description || "No description"}</span></span>
                <ChevronRight aria-hidden="true" className="size-4 shrink-0 opacity-50" />
              </button>)}
              {archivedFeeds.length > 0 && <div className="mt-3 border-t px-2 pt-3"><p className="mb-2 text-[11px] font-semibold uppercase tracking-[0.1em] text-muted-foreground">Archived</p>
                {archivedFeeds.map((feed) => <button className={`flex w-full items-center gap-2 rounded-md px-2 py-2 text-left text-sm ${selectedId === feed.id ? "bg-muted text-foreground" : "text-muted-foreground hover:bg-muted"}`} key={feed.id} onClick={() => selectFeed(feed.id)} type="button"><Archive aria-hidden="true" className="size-3.5" /><span className="min-w-0 flex-1 truncate">{feed.display_name}</span><ChevronRight aria-hidden="true" className="size-3.5" /></button>)}
              </div>}
            </CardContent>
          </Card>

        </aside>
        <section aria-label="Feed details" className="min-w-0">
          {selectedFeed
            ? <FeedDetails feed={selectedFeed} onRequestConfirm={setConfirmAction} onReviewMonth={onReviewMonth} onSaved={() => reloadFeeds(selectedFeed.id)} />
            : <div className="flex min-h-[360px] flex-col items-center justify-center rounded-xl border border-dashed bg-background px-6 text-center"><div className="mb-4 flex size-12 items-center justify-center rounded-xl bg-muted text-muted-foreground"><Rss aria-hidden="true" className="size-5" /></div><h2 className="text-lg font-semibold">Choose a feed</h2><p className="mt-2 max-w-sm text-sm text-muted-foreground">Select an author on the left or create a feed to get started.</p></div>}
        </section>
      </div>

      <AlertDialog open={confirmAction !== null} onOpenChange={(open) => { if (!open) setConfirmAction(null) }}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{confirmAction?.kind === "delete" ? "Permanently delete this feed?" : "Archive this feed?"}</AlertDialogTitle>
            <AlertDialogDescription>
              {confirmAction?.kind === "delete"
                ? `This permanently removes ${confirmAction.feed.display_name}, its years and months, cached PDFs, and refresh history. This cannot be undone.`
                : `${confirmAction?.feed.display_name} and its contents will be hidden from refresh and RSS. You can restore it later.`}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction className={confirmAction?.kind === "delete" ? "bg-destructive text-destructive-foreground hover:bg-destructive/90" : ""} onClick={(event) => { event.preventDefault(); void confirm() }}>
              {confirmAction?.kind === "delete" ? "Delete permanently" : "Archive feed"}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}


export function CreateFeed({ onCreated, onCancel }: { onCreated: (feed: Feed) => void; onCancel: () => void }) {
  async function create(values: { display_name: string; description: string; refresh_interval_seconds: number | null }) {
    const feed = await api.createFeed(values)
    onCreated(feed)
    toast.success("Feed created", { description: `${feed.display_name} is ready for source folders.` })
  }

  return (
    <div className="mx-auto w-full max-w-xl space-y-6">
      <div><h1 className="text-2xl font-semibold tracking-tight">Create a feed</h1><p className="mt-2 text-sm text-muted-foreground">Add the author-facing details for a new channel.</p></div>
      <Card><CardContent className="pt-6"><FeedForm onCancel={onCancel} onSubmit={create} submitLabel="Create feed" /></CardContent></Card>
    </div>
  )
}
