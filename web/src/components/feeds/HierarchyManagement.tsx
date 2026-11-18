import { useCallback, useEffect, useMemo, useRef, useState, type FormEvent } from "react"
import { Archive, CalendarDays, ChevronDown, CheckCircle2, Clock3, FileSearch, FolderPlus, MoreHorizontal, RefreshCw, RotateCcw, Trash2 } from "lucide-react"
import { toast } from "sonner"
import { api, type Month, type Year } from "@/lib/api"
import { formatLocalTimestamp } from "@/lib/datetime"
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
import { DropdownMenu, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu"
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog"
import { Field, FieldGroup, FieldLabel } from "@/components/ui/field"
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from "@/components/ui/empty"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { cn } from "@/lib/utils"
import { Input } from "@/components/ui/input"

const monthNames = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"]
type Confirmation = { title: string; description: string; label: string; destructive?: boolean; action: () => Promise<void> }

function ConfirmDialog({ confirmation, onClose, returnFocus }: { confirmation: Confirmation | null; onClose: () => void; returnFocus?: React.RefObject<HTMLButtonElement | null> }) {
  const [pending, setPending] = useState(false)
  async function confirm(event: React.MouseEvent<HTMLButtonElement>) {
    event.preventDefault()
    if (!confirmation) return
    setPending(true)
    try {
      await confirmation.action()
      onClose()
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "The action could not be completed. Please try again.")
    } finally {
      setPending(false)
    }
  }
  return (
    <AlertDialog open={confirmation !== null} onOpenChange={(open) => { if (!open) onClose() }}>
      <AlertDialogContent onCloseAutoFocus={(event) => { if (returnFocus?.current?.isConnected) { event.preventDefault(); returnFocus.current.focus() } }}>
        <AlertDialogHeader><AlertDialogTitle>{confirmation?.title}</AlertDialogTitle><AlertDialogDescription>{confirmation?.description}</AlertDialogDescription></AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction className={confirmation?.destructive ? "bg-destructive text-destructive-foreground hover:bg-destructive/90" : ""} disabled={pending} onClick={(event) => void confirm(event)}>
            {pending ? "Working…" : confirmation?.label}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}

function MonthCard({ month, year, onChanged, onReview }: {
  month: Month
  year: Year
  onChanged: () => Promise<void>
  onReview: (month: Month) => void
}) {
  const [confirmation, setConfirmation] = useState<Confirmation | null>(null)
  const [fetching, setFetching] = useState(false)
  const [sourceOpen, setSourceOpen] = useState(false)
  const isActive = month.active && year.active
  const monthName = monthNames[month.month - 1] ?? `Month ${month.month}`
  const result = month.last_result?.toLowerCase()

  async function queueRefresh() {
    try {
      await api.refreshMonth(month.id)
      toast.success(`${monthName} refresh queued`)
      await onChanged()
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Could not queue month refresh.")
    }
  }

  async function fetchAndParse() {
    setFetching(true)
    try {
      const result = await api.discoverMonth(month.id)
      const documents = await api.documents(month.id)
      const failed = await Promise.all(documents.map(async (document) => {
        try { await api.extractDocument(document.id); return false } catch { return true }
      }))
      const failures = failed.filter(Boolean).length
      toast.success("Folder scan complete", { description: `${result.discovered_pdfs} PDFs found · ${failures} extraction failures` })
      await onChanged()
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Could not scan this folder.")
    } finally {
      setFetching(false)
    }
  }

  function requestArchive() {
    setConfirmation({
      title: `Archive ${monthName}?`,
      description: "This month and its entries will be hidden from refresh and RSS. You can restore it later.",
      label: "Archive month",
      action: async () => { await api.archiveMonth(month.id); await onChanged(); toast.success(`${monthName} archived`) },
    })
  }

  function requestDelete() {
    setConfirmation({
      title: `Permanently delete ${monthName} ${year.year}?`,
      description: "This permanently removes its source link, cached PDFs, and refresh history. This cannot be undone.",
      label: "Delete permanently",
      destructive: true,
      action: async () => { await api.deleteMonth(month.id); await onChanged(); toast.success(`${monthName} permanently deleted`) },
    })
  }

  async function restore() {
    try { await api.restoreMonth(month.id); await onChanged(); toast.success(`${monthName} restored`) }
    catch (error) { toast.error(error instanceof Error ? error.message : "Could not restore month.") }
  }

  return (
    <Card className="min-w-0 shadow-none">
      <CardHeader className="flex min-w-0 flex-row flex-wrap items-center justify-between gap-3">
          <div className="min-w-0">
            <div className="flex flex-wrap items-center gap-2"><h4 className="text-sm font-semibold">{monthName}</h4><Badge variant={isActive ? "secondary" : "outline"}>{isActive ? "Active" : "Archived"}</Badge></div>

          </div>
          <div className="flex flex-wrap gap-2"><Badge variant={result === "success" ? "success" : result === "failure" ? "destructive" : "outline"}>{month.last_result ?? "Not refreshed"}</Badge><Button aria-controls={`source-${month.id}`} aria-expanded={sourceOpen} onClick={() => setSourceOpen((open) => !open)} size="sm" variant="outline">Manage source</Button><Button onClick={() => onReview(month)} size="sm" variant="ghost">Review PDFs</Button></div>
      </CardHeader>
      <CardContent className="flex min-w-0 flex-col gap-4" hidden={!sourceOpen} id={`source-${month.id}`}>
            <a className="mt-1.5 block break-all text-xs text-primary underline-offset-4 hover:underline" href={month.source_url} rel="noreferrer" target="_blank">{month.source_url}</a>
        <section aria-label={`${monthName} refresh status`} className="grid min-w-0 gap-3 rounded-lg bg-muted/45 p-3 sm:grid-cols-3">
          <div className="min-w-0"><p className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground"><Clock3 aria-hidden="true" className="size-3" />Last attempt</p><p className="mt-1 break-words text-xs">{formatLocalTimestamp(month.last_attempt_at, "Never")}</p></div>
          <div className="min-w-0"><p className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground"><CalendarDays aria-hidden="true" className="size-3" />Next refresh</p><p className="mt-1 break-words text-xs">{formatLocalTimestamp(month.next_refresh_at, "Pending")}</p></div>
          <div className="min-w-0"><p className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground"><CheckCircle2 aria-hidden="true" className="size-3" />Latest result</p><p className="mt-1 break-words text-xs">{month.last_error ? `Failed · ${month.last_error}` : month.last_result ?? "Not run yet"}</p></div>
        </section>

        <div className="flex flex-wrap items-center gap-2 border-t pt-3">
          {isActive && <>
            <Button disabled={fetching} onClick={() => void fetchAndParse()} size="sm" variant="outline"><FileSearch aria-hidden="true" className={fetching ? "animate-pulse" : ""} />{fetching ? "Scanning…" : "Fetch and parse"}</Button>
            <Button onClick={() => void queueRefresh()} size="sm" variant="outline"><RefreshCw aria-hidden="true" />Refresh now</Button>

            <Button onClick={requestArchive} size="sm" variant="ghost"><Archive aria-hidden="true" />Archive</Button>
          </>}
          {!month.active && year.active && <Button onClick={() => void restore()} size="sm" variant="outline"><RotateCcw aria-hidden="true" />Restore month</Button>}
          <Button className="sm:ml-auto" onClick={requestDelete} size="sm" variant="ghost"><Trash2 aria-hidden="true" className="text-destructive" /><span className="text-destructive">Delete permanently</span></Button>
        </div>
      </CardContent>
      <ConfirmDialog confirmation={confirmation} onClose={() => setConfirmation(null)} />
    </Card>
  )
}

function YearCard({ year, feedId, onChanged, onReview }: {
  year: Year
  feedId: string
  onChanged: () => Promise<void>
  onReview: (month: Month) => void
}) {
  const storageKey = `kpapers.year.expanded.v1.${feedId}.${year.year}`
  const [expanded, setExpanded] = useState(() => {
    try { return window.localStorage.getItem(storageKey) === "expanded" } catch { return false }
  })
  const regionId = `year-content-${feedId}-${year.year}`
  const actionsRef = useRef<HTMLButtonElement>(null)

  function toggleExpanded() {
    const next = !expanded
    setExpanded(next)
    try { window.localStorage.setItem(storageKey, next ? "expanded" : "collapsed") } catch { /* Keep toggling usable when storage is blocked. */ }
  }
  const [months, setMonths] = useState<Month[]>([])
  const [loading, setLoading] = useState(true)
  const [formError, setFormError] = useState<string | null>(null)
  const [confirmation, setConfirmation] = useState<Confirmation | null>(null)
  const [monthValue, setMonthValue] = useState("")
  const [connecting, setConnecting] = useState(false)
  const [linking, setLinking] = useState(false)
  const [sourceUrl, setSourceUrl] = useState("")

  const reloadMonths = useCallback(async () => {
    setLoading(true)
    try { setMonths(await api.months(feedId, year.year)) }
    catch (error) { toast.error(error instanceof Error ? error.message : "Could not load configured months.") }
    finally { setLoading(false) }
  }, [feedId, year.year])

  useEffect(() => {
    let mounted = true
    api.months(feedId, year.year).then((items) => { if (mounted) setMonths(items) }).catch((error: unknown) => {
      if (mounted) toast.error(error instanceof Error ? error.message : "Could not load configured months.")
    }).finally(() => { if (mounted) setLoading(false) })
    return () => { mounted = false }
  }, [feedId, year.year])

  const availableMonths = useMemo(() => monthNames.map((name, index) => ({ name, value: index + 1 })).filter((item) => !months.some((month) => month.month === item.value)), [months])

  async function createMonth(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (!monthValue || !sourceUrl) {
      setFormError("Choose a month and enter its public folder link.")
      return
    }
    setFormError(null)
    setLinking(true)
    try {
      await api.createMonth(feedId, year.year, Number(monthValue), sourceUrl.trim())
      setMonthValue("")
      setSourceUrl("")
      await reloadMonths()
      setConnecting(false)
      toast.success(`${monthNames[Number(monthValue) - 1]} linked`)
    } catch (error) {
      setFormError(error instanceof Error ? error.message : "Could not link this month.")
    } finally { setLinking(false) }
  }

  async function refreshYear() {
    try { await api.refreshYear(feedId, year.year); toast.success(`${year.year} refresh queued`) }
    catch (error) { toast.error(error instanceof Error ? error.message : "Could not queue year refresh.") }
  }

  function requestArchive() {
    setConfirmation({
      title: `Archive ${year.year}?`,
      description: "This year and its months will be hidden from refresh and RSS until restored.",
      label: "Archive year",
      action: async () => { await api.archiveYear(feedId, year.year); await onChanged(); toast.success(`${year.year} archived`) },
    })
  }

  function requestDelete() {
    setConfirmation({
      title: `Permanently delete ${year.year}?`,
      description: "This permanently removes the year, its months, cached PDFs, and refresh history. This cannot be undone.",
      label: "Delete permanently",
      destructive: true,
      action: async () => { await api.deleteYear(feedId, year.year); await onChanged(); toast.success(`${year.year} permanently deleted`) },
    })
  }

  async function restoreYear() {
    try { await api.restoreYear(feedId, year.year); await onChanged(); toast.success(`${year.year} restored`) }
    catch (error) { toast.error(error instanceof Error ? error.message : "Could not restore year.") }
  }

  return (
    <Card className="min-w-0">
      <CardHeader className="flex flex-row flex-wrap items-center justify-between gap-3 pb-4">
        <div className="flex items-center gap-3"><div className="flex size-9 items-center justify-center rounded-lg bg-muted text-muted-foreground"><CalendarDays aria-hidden="true" className="size-4" /></div><div><div className="flex items-center gap-2"><CardTitle as="h3">{year.year}</CardTitle><Badge variant={year.active ? "secondary" : "outline"}>{year.active ? "Active" : "Archived"}</Badge></div><CardDescription className="mt-0.5">{months.length} configured {months.length === 1 ? "month" : "months"}</CardDescription></div></div>
        <div className="flex flex-wrap gap-2">
          <Button aria-controls={regionId} aria-expanded={expanded} aria-label={`${expanded ? "Collapse" : "Expand"} ${year.year}`} onClick={toggleExpanded} size="sm" variant="outline"><ChevronDown aria-hidden="true" className={cn(!expanded && "-rotate-90")} />{expanded ? "Collapse" : "Expand"}</Button>
          <DropdownMenu><DropdownMenuTrigger asChild><Button ref={actionsRef} aria-label={`Actions for ${year.year}`} size="icon" variant="ghost"><MoreHorizontal aria-hidden="true" /></Button></DropdownMenuTrigger><DropdownMenuContent align="end"><DropdownMenuGroup>
            {year.active ? <><DropdownMenuItem onSelect={() => void refreshYear()}><RefreshCw aria-hidden="true" />Refresh year</DropdownMenuItem><DropdownMenuItem onSelect={requestArchive}><Archive aria-hidden="true" />Archive year</DropdownMenuItem></> : <DropdownMenuItem onSelect={() => void restoreYear()}><RotateCcw aria-hidden="true" />Restore year</DropdownMenuItem>}
            <DropdownMenuItem variant="destructive" onSelect={requestDelete}><Trash2 aria-hidden="true" />Delete year permanently</DropdownMenuItem>
          </DropdownMenuGroup></DropdownMenuContent></DropdownMenu>
        </div>
      </CardHeader>
      <CardContent className="flex flex-col gap-5" hidden={!expanded} id={regionId}>
        {year.active && <Dialog open={connecting} onOpenChange={setConnecting}><DialogTrigger asChild><Button className="self-start" disabled={availableMonths.length === 0} variant="outline"><FolderPlus aria-hidden="true" data-icon="inline-start" />Connect month</Button></DialogTrigger><DialogContent><DialogHeader><DialogTitle>Connect month in {year.year}</DialogTitle><DialogDescription>Add a public cloud folder containing PDF issues.</DialogDescription></DialogHeader>
          <form className="flex flex-col gap-4" onSubmit={(event) => void createMonth(event)}><FieldGroup>
            <Field><FieldLabel htmlFor={`month-${year.year}`}>Month</FieldLabel><select className="w-full rounded-lg border bg-card px-3 text-sm" id={`month-${year.year}`} onChange={(event) => setMonthValue(event.target.value)} required value={monthValue}><option value="">Choose month</option>{availableMonths.map(({name, value}) => <option key={value} value={value}>{name}</option>)}</select></Field>
            <Field><FieldLabel htmlFor={`source-${year.year}`}>Public folder link</FieldLabel><Input id={`source-${year.year}`} onChange={(event) => setSourceUrl(event.target.value)} placeholder="https://disk.yandex.ru/... or cloud.mail.ru/..." required type="url" value={sourceUrl} /></Field>
          </FieldGroup>{formError && <Alert variant="destructive"><AlertDescription>{formError}</AlertDescription></Alert>}
          <div className="flex flex-col gap-2 sm:flex-row sm:justify-end"><Button onClick={() => setConnecting(false)} type="button" variant="outline">Cancel</Button><Button disabled={linking} type="submit">{linking ? "Connecting…" : "Link month"}</Button></div></form>
        </DialogContent></Dialog>}

        <div className="flex flex-col gap-3">
          <div className="flex items-center justify-between"><h3 className="text-sm font-medium">Configured months</h3>{loading && <span className="text-xs text-muted-foreground">Loading…</span>}</div>
          {months.length === 0 && !loading && <div className="rounded-lg border border-dashed px-4 py-6 text-center text-sm text-muted-foreground">No months linked yet. Months without folders can stay empty.</div>}
          {months.map((month) => <MonthCard key={month.id} month={month} onChanged={reloadMonths} onReview={onReview} year={year} />)}
        </div>
      </CardContent>
      <ConfirmDialog confirmation={confirmation} onClose={() => setConfirmation(null)} returnFocus={actionsRef} />
    </Card>
  )
}

export function HierarchyManagement({ feedId, active, onReviewMonth }: {
  feedId: string
  active: boolean
  onReviewMonth: (month: Month) => void
}) {
  const [years, setYears] = useState<Year[]>([])
  const [loading, setLoading] = useState(true)
  const [yearValue, setYearValue] = useState("")
  const [adding, setAdding] = useState(false)
  const [pending, setPending] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const reloadYears = useCallback(async () => {
    try { setYears(await api.years(feedId)) }
    catch (cause) { toast.error(cause instanceof Error ? cause.message : "Could not load years.") }
  }, [feedId])

  useEffect(() => {
    let mounted = true
    api.years(feedId).then((items) => { if (mounted) setYears(items) }).catch((cause: unknown) => {
      if (mounted) toast.error(cause instanceof Error ? cause.message : "Could not load years.")
    }).finally(() => { if (mounted) setLoading(false) })
    return () => { mounted = false }
  }, [feedId])

  async function createYear(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const value = Number(yearValue)
    if (!Number.isInteger(value) || value < 1 || value > 9999) {
      setError("Enter a year between 1 and 9999.")
      return
    }
    setError(null)
    setPending(true)
    try {
      await api.createYear(feedId, value)
      setYearValue("")
      await reloadYears()
      setAdding(false)
      toast.success(`${value} added`)
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Could not add year.")
    } finally { setPending(false) }
  }

  return (
    <Card className="min-w-0">
      <CardHeader className="flex flex-row flex-wrap items-start justify-between gap-4"><div><CardTitle as="h2" className="text-base">Source folders</CardTitle><CardDescription className="mt-1">Organize documents by year and connect the months you publish.</CardDescription></div>
        <Dialog open={adding} onOpenChange={setAdding}><DialogTrigger asChild><Button disabled={!active} variant="outline"><FolderPlus aria-hidden="true" data-icon="inline-start" />Add year</Button></DialogTrigger><DialogContent><DialogHeader><DialogTitle>Add year</DialogTitle><DialogDescription>Create a year before connecting its source folders.</DialogDescription></DialogHeader>
          <form className="flex flex-col gap-4" onSubmit={(event) => void createYear(event)}><FieldGroup><Field><FieldLabel htmlFor={`new-year-${feedId}`}>Add a year</FieldLabel><Input id={`new-year-${feedId}`} min="1" max="9999" onChange={(event) => setYearValue(event.target.value)} placeholder="e.g. 2026" required type="number" value={yearValue} /></Field></FieldGroup>{error && <Alert variant="destructive"><AlertDescription>{error}</AlertDescription></Alert>}<div className="flex flex-col gap-2 sm:flex-row sm:justify-end"><Button onClick={() => setAdding(false)} type="button" variant="outline">Cancel</Button><Button disabled={pending} type="submit">{pending ? "Adding…" : "Add year"}</Button></div></form>
        </DialogContent></Dialog>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        {years.map((year) => <YearCard key={`${feedId}-${year.year}`} feedId={feedId} onChanged={reloadYears} onReview={onReviewMonth} year={year} />)}
        {years.length === 0 && !loading && <Empty><EmptyHeader><EmptyTitle>No years configured</EmptyTitle><EmptyDescription>Add a year to start linking source folders.</EmptyDescription></EmptyHeader></Empty>}
        {loading && <div aria-label="Loading years" className="rounded-xl border bg-background px-6 py-8 text-center text-sm text-muted-foreground">Loading years…</div>}
      </CardContent>
    </Card>
  )
}
