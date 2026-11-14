import { useMemo, useRef, useState, type FormEvent } from "react"
import { Archive, ArrowLeft, Check, ChevronRight, Copy, FileText, MoreHorizontal, Plus, RefreshCw, Rss, Trash2, Undo2 } from "lucide-react"
import { toast } from "sonner"
import { api, type ExtractionEngine, type Feed, type Month } from "@/lib/api"
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
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog"
import { DropdownMenu, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Field, FieldDescription, FieldError, FieldGroup, FieldLabel } from "@/components/ui/field"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty"
import { Input } from "@/components/ui/input"
import { Textarea } from "@/components/ui/textarea"
import { HierarchyManagement } from "@/components/feeds/HierarchyManagement"

type FeedFormValues = { display_name: string; description: string; refresh_hours: string; extraction_engine: "inherit" | ExtractionEngine }
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
  onSubmit: (values: { display_name: string; description: string; refresh_interval_seconds: number | null; extraction_engine: ExtractionEngine | null }) => Promise<void>
  onCancel?: () => void
}) {
  const [values, setValues] = useState<FeedFormValues>(() => ({
    display_name: initial?.display_name ?? "",
    description: initial?.description ?? "",
    refresh_hours: hoursFromSeconds(initial?.refresh_interval_seconds ?? null),
    extraction_engine: initial?.extraction_engine ?? "inherit",
  }))
  const [pending, setPending] = useState(false)
  const [fieldErrors, setFieldErrors] = useState<Partial<Record<keyof FeedFormValues, string>>>({})
  const summaryRef = useRef<HTMLDivElement>(null)
  function showErrors(errors: Partial<Record<keyof FeedFormValues, string>>) {
    setFieldErrors(errors)
    setError("Check the highlighted fields.")
    requestAnimationFrame(() => summaryRef.current?.focus())
  }
  const [error, setError] = useState<string | null>(null)

  function update(name: keyof FeedFormValues, value: string) {
    setValues((current) => ({ ...current, [name]: value }))
    setError(null)
    setFieldErrors({})
  }

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const name = values.display_name.trim()
    const invalid: Partial<Record<keyof FeedFormValues, string>> = {}
    if (!name) invalid.display_name = "Enter an author name."
    const hours = Number(values.refresh_hours)
    if (values.refresh_hours && (!Number.isFinite(hours) || hours <= 0)) invalid.refresh_hours = "Refresh interval must be greater than zero hours."
    else if (hours > 87600) invalid.refresh_hours = "Refresh interval must be at most 87600 hours."
    else if (values.refresh_hours && Math.abs(hours * 10 - Math.round(hours * 10)) > 0.000001) invalid.refresh_hours = "Use intervals in steps of 0.1 hours."
    if (Object.keys(invalid).length) { showErrors(invalid); return }
    setFieldErrors({})
    setError(null)
    setPending(true)
    try {
      await onSubmit({
        display_name: name,
        description: values.description.trim(),
        refresh_interval_seconds: secondsFromHours(values.refresh_hours),
        extraction_engine: values.extraction_engine === "inherit" ? null : values.extraction_engine,
      })
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Could not save feed. Please try again.")
      requestAnimationFrame(() => summaryRef.current?.focus())
    } finally {
      setPending(false)
    }
  }

  const id = initial ? `edit-feed-${initial.id}` : "create-feed"
  return (
    <form className="flex flex-col gap-6" noValidate onSubmit={(event) => void submit(event)}>
      {error && <Alert data-testid="feed-error-summary" ref={summaryRef} tabIndex={-1} variant="destructive"><AlertTitle>Could not save feed</AlertTitle><AlertDescription>{Object.keys(fieldErrors).length ? <ul className="flex flex-col gap-1">{Object.entries(fieldErrors).map(([field, message]) => <li key={field}><a className="underline" href={`#${id}-${field}`} onClick={(event) => { event.preventDefault(); document.getElementById(`${id}-${field}`)?.focus() }}>{message}</a></li>)}</ul> : error}</AlertDescription></Alert>}
      <FieldGroup>
        <Field data-invalid={Boolean(fieldErrors.display_name)}><FieldLabel htmlFor={`${id}-display_name`}>Author name</FieldLabel><Input aria-describedby={fieldErrors.display_name ? `${id}-name-error` : undefined} aria-invalid={Boolean(fieldErrors.display_name)} autoComplete="off" id={`${id}-display_name`} name="display_name" onChange={(event) => update("display_name", event.target.value)} placeholder="e.g. Ada Lovelace" required value={values.display_name} />{fieldErrors.display_name && <FieldError id={`${id}-name-error`}>{fieldErrors.display_name}</FieldError>}</Field>
        <Field><FieldLabel htmlFor={`${id}-description`}>Description <span className="font-normal text-muted-foreground">(optional)</span></FieldLabel><Textarea id={`${id}-description`} name="description" onChange={(event) => update("description", event.target.value)} placeholder="A short introduction for readers" rows={4} value={values.description} /></Field>
        <Field data-invalid={Boolean(fieldErrors.refresh_hours)}><FieldLabel htmlFor={`${id}-refresh_hours`}>Refresh interval <span className="font-normal text-muted-foreground">(optional)</span></FieldLabel><Input aria-describedby={`${id}-refresh-help${fieldErrors.refresh_hours ? ` ${id}-refresh-error` : ""}`} aria-invalid={Boolean(fieldErrors.refresh_hours)} id={`${id}-refresh_hours`} max="87600" name="refresh_hours" onChange={(event) => update("refresh_hours", event.target.value)} placeholder="Use defaults" step="0.1" type="number" value={values.refresh_hours} /><FieldDescription id={`${id}-refresh-help`}>In hours. Leave blank to use the current and older month defaults.</FieldDescription>{fieldErrors.refresh_hours && <FieldError id={`${id}-refresh-error`}>{fieldErrors.refresh_hours}</FieldError>}</Field>
        <Field><FieldLabel htmlFor={`${id}-extraction_engine`}>PDF extraction engine</FieldLabel><select aria-describedby={`${id}-engine-help`} className="h-9 w-full rounded-md border border-input bg-background px-3 text-sm font-normal" id={`${id}-extraction_engine`} name="extraction_engine" onChange={(event) => update("extraction_engine", event.target.value)} value={values.extraction_engine}><option value="inherit">Inherit global default</option><option value="pdfium">Pdfium</option><option value="marker">Marker</option></select><FieldDescription id={`${id}-engine-help`}>Marker requires the optional private worker. Inherit follows the deployment-wide default.</FieldDescription></Field>
      </FieldGroup>
      <div className="flex flex-col gap-2 sm:flex-row sm:justify-end">
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

  async function save(values: { display_name: string; description: string; refresh_interval_seconds: number | null; extraction_engine: ExtractionEngine | null }) {
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
    <div className="flex flex-col gap-5">
      <Card>
        <CardHeader className="flex flex-row flex-wrap items-start justify-between gap-4">
          <div className="flex min-w-0 gap-3">
            <div className="flex size-11 shrink-0 items-center justify-center rounded-xl bg-primary/8 text-primary"><Rss aria-hidden="true" className="size-5" /></div>
            <div className="min-w-0"><div className="flex flex-wrap items-center gap-2"><h1>{feed.display_name}</h1><Badge variant="secondary">{feed.active ? "Active" : "Archived"}</Badge></div><CardDescription className="mt-1.5 max-w-2xl">{feed.description || "No description added yet."}</CardDescription></div>
          </div>
          {feed.active && <div className="flex flex-wrap gap-2">

            <Dialog open={editing} onOpenChange={setEditing}><DialogTrigger asChild><Button variant="outline"><FileText aria-hidden="true" data-icon="inline-start" />Edit details</Button></DialogTrigger><DialogContent><DialogHeader><DialogTitle>Edit feed details</DialogTitle><DialogDescription>Changes to the display name do not change the RSS address.</DialogDescription></DialogHeader><FeedForm initial={feed} onCancel={() => setEditing(false)} onSubmit={save} submitLabel="Save changes" /></DialogContent></Dialog>
          </div>}
          {!feed.active && <div className="flex flex-wrap gap-2">
            <Button onClick={async () => { try { await api.restoreFeed(feed.id); await onSaved(); toast.success("Feed restored") } catch (error) { toast.error(error instanceof Error ? error.message : "Could not restore feed.") } }} variant="outline"><Undo2 aria-hidden="true" /> Restore</Button>
            <Button onClick={() => onRequestConfirm({ kind: "delete", feed })} variant="destructive"><Trash2 aria-hidden="true" /> Delete permanently</Button>
          </div>}
          {feed.active && <DropdownMenu><DropdownMenuTrigger asChild><Button aria-label={`More actions for ${feed.display_name}`} size="icon" variant="outline"><MoreHorizontal aria-hidden="true" /></Button></DropdownMenuTrigger><DropdownMenuContent align="end"><DropdownMenuGroup>{feed.active && <><DropdownMenuItem disabled={refreshing} onSelect={() => void refresh()}><RefreshCw aria-hidden="true" />Refresh feed</DropdownMenuItem><DropdownMenuItem variant="destructive" onSelect={() => onRequestConfirm({ kind: "archive", feed })}><Archive aria-hidden="true" />Archive feed</DropdownMenuItem></>}</DropdownMenuGroup></DropdownMenuContent></DropdownMenu>}
        </CardHeader>
        <CardContent className="border-t pt-4">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="min-w-0"><p className="text-xs font-medium text-muted-foreground">Stable RSS address</p><code className="mt-1 block break-all rounded bg-muted px-2 py-1 text-xs text-foreground">{rssUrl}</code></div>
            <Button aria-label="Copy RSS address" onClick={() => void copyUrl()} size="sm" variant="outline">{copied ? <Check aria-hidden="true" /> : <Copy aria-hidden="true" />} {copied ? "Copied" : "Copy link"}</Button>
          </div>
        </CardContent>
      </Card>


      <HierarchyManagement active={feed.active} feedId={feed.id} onReviewMonth={onReviewMonth} />

      <div className="flex flex-wrap gap-x-5 gap-y-2 text-xs text-muted-foreground">
        <span>Feed ID <code className="ml-1 break-all text-[11px] text-foreground/70">{feed.id}</code></span>
        <span>Refresh interval {feed.refresh_interval_seconds ? `${(feed.refresh_interval_seconds / 3600).toLocaleString()} hours` : "Default schedule"}</span>
        <span>Extraction engine {feed.extraction_engine === "marker" ? "Marker" : feed.extraction_engine === "pdfium" ? "Pdfium" : "Inherit global default"}</span>
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
  const [localSelectedId, setLocalSelectedId] = useState<string | null>(null)
  const selectedId = selectedFeedId === undefined ? localSelectedId : selectedFeedId
  const [confirmAction, setConfirmAction] = useState<ConfirmAction | null>(null)
  const [confirmPending, setConfirmPending] = useState(false)
  const confirmTrigger = useRef<HTMLElement | null>(null)
  function requestConfirm(action: ConfirmAction) {
    const label = selectedFeedId ? `More actions for ${action.feed.display_name}` : `Actions for ${action.feed.display_name}`
    confirmTrigger.current = Array.from(document.querySelectorAll("button")).find((button) => button.getAttribute("aria-label") === label) ?? document.activeElement as HTMLElement
    setConfirmAction(action)
  }
  const selectedFeed = feeds.find((feed) => feed.id === selectedId) ?? null

  function selectFeed(id: string | null) {
    setLocalSelectedId(id)
    onSelectFeed?.(id)
  }


  async function reloadFeeds(selectId?: string) {
    const loaded = await api.feeds()
    onFeedsChange(loaded)
    if (selectId) selectFeed(selectId)
    else if (selectedId && !loaded.some((feed) => feed.id === selectedId)) selectFeed(null)
  }

  async function confirm() {
    if (!confirmAction) return
    const { kind, feed } = confirmAction
    setConfirmPending(true)
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
    } finally { setConfirmPending(false) }
  }

  return (
    <div className="flex flex-col gap-6">
      {selectedFeed ? <section aria-label="Feed details" className="flex min-w-0 flex-col gap-5">
        <Button className="self-start" onClick={() => selectFeed(null)} variant="ghost"><ArrowLeft aria-hidden="true" data-icon="inline-start" />All feeds</Button>
        <FeedDetails key={selectedFeed.id} feed={selectedFeed} onRequestConfirm={requestConfirm} onReviewMonth={onReviewMonth} onSaved={() => reloadFeeds(selectedFeed.id)} />
      </section> : <>
        <div className="flex flex-wrap items-start justify-between gap-4"><div><h1>Feeds</h1><p className="mt-2 text-sm text-muted-foreground">Manage author feeds and their source folders.</p></div><Button onClick={onCreate}><Plus aria-hidden="true" data-icon="inline-start" />Create feed</Button></div>
        <Tabs defaultValue="active"><TabsList aria-label="Feed status"><TabsTrigger value="active">Active ({activeFeeds.length})</TabsTrigger><TabsTrigger value="archived">Archived ({archivedFeeds.length})</TabsTrigger></TabsList>
          {[{value: "active", items: activeFeeds}, {value: "archived", items: archivedFeeds}].map(({value, items}) => <TabsContent key={value} value={value}>
            <Card><CardHeader><CardTitle>{value === "active" ? "Active feeds" : "Archived feeds"}</CardTitle><CardDescription>Open a feed to manage its details and source folders.</CardDescription></CardHeader><CardContent className="flex flex-col gap-2">
              {items.map((feed) => <div className="flex min-w-0 flex-wrap items-center gap-3 rounded-lg border p-4" key={feed.id}>
                <Button className="h-auto min-w-0 flex-1 justify-start gap-3 whitespace-normal px-0 text-left" onClick={() => selectFeed(feed.id)} variant="ghost"><Rss aria-hidden="true" data-icon="inline-start" /><span className="min-w-0 flex-1"><span className="block break-words">{feed.display_name}</span><span className="mt-1 block break-words text-sm font-normal text-muted-foreground">{feed.description || "No description"}</span></span><span className="hidden sm:inline">Open feed</span><ChevronRight aria-hidden="true" data-icon="inline-end" /></Button>
                <Badge variant={feed.active ? "success" : "outline"}>{feed.active ? "Active" : "Archived"}</Badge>
                {!feed.active && <DropdownMenu><DropdownMenuTrigger asChild><Button aria-label={`Actions for ${feed.display_name}`} size="icon" variant="ghost"><MoreHorizontal aria-hidden="true" /></Button></DropdownMenuTrigger><DropdownMenuContent align="end"><DropdownMenuGroup><DropdownMenuItem onSelect={async () => { try { await api.restoreFeed(feed.id); await reloadFeeds(); toast.success("Feed restored") } catch (error) { toast.error(error instanceof Error ? error.message : "Could not restore feed.") } }}><Undo2 aria-hidden="true" />Restore</DropdownMenuItem><DropdownMenuItem variant="destructive" onSelect={() => requestConfirm({kind: "delete", feed})}><Trash2 aria-hidden="true" />Delete permanently</DropdownMenuItem></DropdownMenuGroup></DropdownMenuContent></DropdownMenu>}
              </div>)}
              {items.length === 0 && <Empty><EmptyHeader><EmptyMedia variant="icon"><Rss aria-hidden="true" /></EmptyMedia><EmptyTitle>{value === "active" ? "No active feeds yet" : "No archived feeds"}</EmptyTitle><EmptyDescription>{value === "active" ? "Create an author feed to start connecting source folders." : "Feeds you archive will appear here."}</EmptyDescription></EmptyHeader>{value === "active" && <EmptyContent><Button onClick={onCreate}>Create your first feed</Button></EmptyContent>}</Empty>}
            </CardContent></Card>
          </TabsContent>)}
        </Tabs>
      </>}

      <AlertDialog open={confirmAction !== null} onOpenChange={(open) => { if (!open) setConfirmAction(null) }}>
        <AlertDialogContent onCloseAutoFocus={(event) => { if (confirmTrigger.current?.isConnected) { event.preventDefault(); confirmTrigger.current.focus() } }}>
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
            <AlertDialogAction disabled={confirmPending} className={confirmAction?.kind === "delete" ? "bg-destructive text-destructive-foreground hover:bg-destructive/90" : ""} onClick={(event) => { event.preventDefault(); void confirm() }}>
              {confirmAction?.kind === "delete" ? "Delete permanently" : "Archive feed"}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}


export function CreateFeed({ onCreated, onCancel }: { onCreated: (feed: Feed) => void; onCancel: () => void }) {
  async function create(values: { display_name: string; description: string; refresh_interval_seconds: number | null; extraction_engine: ExtractionEngine | null }) {
    const feed = await api.createFeed(values)
    onCreated(feed)
    toast.success("Feed created", { description: `${feed.display_name} is ready for source folders.` })
  }

  return (
    <div className="mx-auto w-full max-w-[640px] flex flex-col gap-6">
      <div><h1>Create a feed</h1><p className="mt-2 text-sm text-muted-foreground">Add the author-facing details for a new channel.</p></div>
      <Card><CardHeader><CardTitle>Feed details</CardTitle><CardDescription>Connect source folders after creating your feed.</CardDescription></CardHeader><CardContent><FeedForm onCancel={onCancel} onSubmit={create} submitLabel="Create feed" /></CardContent></Card>
    </div>
  )
}
