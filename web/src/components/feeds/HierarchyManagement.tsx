import { useCallback, useEffect, useMemo, useState, type FormEvent } from "react"
import { Archive, CalendarDays, ChevronDown, CheckCircle2, Clock3, FileSearch, FolderPlus, RefreshCw, RotateCcw, Trash2 } from "lucide-react"
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
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"

const monthNames = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"]
type Confirmation = { title: string; description: string; label: string; destructive?: boolean; action: () => Promise<void> }

function ConfirmDialog({ confirmation, onClose }: { confirmation: Confirmation | null; onClose: () => void }) {
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
      <AlertDialogContent>
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
      <CardContent className="grid min-w-0 gap-4 p-4 sm:p-5">
        <div className="flex min-w-0 flex-wrap items-start justify-between gap-3">
          <div className="min-w-0">
            <div className="flex flex-wrap items-center gap-2"><h4 className="text-sm font-semibold">{monthName}</h4><Badge variant={isActive ? "secondary" : "outline"}>{isActive ? "Active" : "Archived"}</Badge></div>
            <a className="mt-1.5 block break-all text-xs text-primary underline-offset-4 hover:underline" href={month.source_url} rel="noreferrer" target="_blank">{month.source_url}</a>
          </div>
          <span className="flex items-center gap-1.5 text-xs text-muted-foreground"><span className={`size-1.5 rounded-full ${result === "success" ? "bg-emerald-500" : result === "failure" ? "bg-destructive" : "bg-muted-foreground/40"}`} />{month.last_result ?? "Not refreshed"}</span>
        </div>

        <section aria-label={`${monthName} refresh status`} className="grid min-w-0 gap-3 rounded-lg bg-muted/45 p-3 sm:grid-cols-3">
          <div className="min-w-0"><p className="flex items-center gap-1.5 text-[11px] font-medium text-muted-foreground"><Clock3 aria-hidden="true" className="size-3" />Last attempt</p><p className="mt-1 break-words text-xs">{formatLocalTimestamp(month.last_attempt_at, "Never")}</p></div>
          <div className="min-w-0"><p className="flex items-center gap-1.5 text-[11px] font-medium text-muted-foreground"><CalendarDays aria-hidden="true" className="size-3" />Next refresh</p><p className="mt-1 break-words text-xs">{formatLocalTimestamp(month.next_refresh_at, "Pending")}</p></div>
          <div className="min-w-0"><p className="flex items-center gap-1.5 text-[11px] font-medium text-muted-foreground"><CheckCircle2 aria-hidden="true" className="size-3" />Latest result</p><p className="mt-1 break-words text-xs">{month.last_error ? `Failed · ${month.last_error}` : month.last_result ?? "Not run yet"}</p></div>
        </section>

        <div className="flex flex-wrap items-center gap-2 border-t pt-3">
          {isActive && <>
            <Button disabled={fetching} onClick={() => void fetchAndParse()} size="sm" variant="outline"><FileSearch aria-hidden="true" className={fetching ? "animate-pulse" : ""} />{fetching ? "Scanning…" : "Fetch and parse"}</Button>
            <Button onClick={() => void queueRefresh()} size="sm" variant="outline"><RefreshCw aria-hidden="true" />Refresh now</Button>
            <Button onClick={() => onReview(month)} size="sm" variant="outline">Review PDFs</Button>
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
    try {
      await api.createMonth(feedId, year.year, Number(monthValue), sourceUrl.trim())
      setMonthValue("")
      setSourceUrl("")
      await reloadMonths()
      toast.success(`${monthNames[Number(monthValue) - 1]} linked`)
    } catch (error) {
      setFormError(error instanceof Error ? error.message : "Could not link this month.")
    }
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
        <div className="flex items-center gap-3"><div className="flex size-9 items-center justify-center rounded-lg bg-muted text-muted-foreground"><CalendarDays aria-hidden="true" className="size-4" /></div><div><div className="flex items-center gap-2"><CardTitle className="text-base">{year.year}</CardTitle><Badge variant={year.active ? "secondary" : "outline"}>{year.active ? "Active" : "Archived"}</Badge></div><CardDescription className="mt-0.5">{months.length} configured {months.length === 1 ? "month" : "months"}</CardDescription></div></div>
        <div className="flex flex-wrap gap-2">
          <Button aria-controls={regionId} aria-expanded={expanded} aria-label={`${expanded ? "Collapse" : "Expand"} ${year.year}`} onClick={toggleExpanded} size="sm" variant="outline"><ChevronDown aria-hidden="true" className={expanded ? "size-4" : "size-4 -rotate-90"} />{expanded ? "Collapse" : "Expand"}</Button>
          {year.active
            ? <><Button onClick={() => void refreshYear()} size="sm" variant="outline"><RefreshCw aria-hidden="true" />Refresh year</Button><Button onClick={requestArchive} size="sm" variant="ghost"><Archive aria-hidden="true" />Archive</Button></>
            : <Button onClick={() => void restoreYear()} size="sm" variant="outline"><RotateCcw aria-hidden="true" />Restore year</Button>}
          <Button aria-label={`Delete ${year.year} permanently`} onClick={requestDelete} size="sm" variant="ghost"><Trash2 aria-hidden="true" className="text-destructive" /><span className="text-destructive">Delete</span></Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-5" hidden={!expanded} id={regionId}>
        <section aria-label={`Link a month in ${year.year}`} className="rounded-xl border bg-muted/20 p-4">
          <div className="mb-3 flex items-start gap-2.5"><div className="mt-0.5 flex size-7 items-center justify-center rounded-lg bg-background text-muted-foreground"><FolderPlus aria-hidden="true" className="size-3.5" /></div><div><h3 className="text-sm font-medium">Link a month</h3><p className="mt-0.5 text-xs text-muted-foreground">Add a public cloud folder when it contains PDF issues.</p></div></div>
          <form className="grid min-w-0 items-end gap-3 md:grid-cols-[minmax(140px,0.75fr)_minmax(220px,2fr)_auto]" onSubmit={(event) => void createMonth(event)}>
            <div className="space-y-1.5"><Label htmlFor={`month-${year.year}`}>Month</Label><select className="h-9 w-full rounded-md border border-input bg-background px-3 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50" disabled={!year.active || availableMonths.length === 0} id={`month-${year.year}`} onChange={(event) => setMonthValue(event.target.value)} required value={monthValue}><option value="">Choose month</option>{availableMonths.map(({ name, value }) => <option key={value} value={value}>{name}</option>)}</select></div>
            <div className="space-y-1.5"><Label htmlFor={`source-${year.year}`}>Public folder link</Label><Input disabled={!year.active} id={`source-${year.year}`} onChange={(event) => setSourceUrl(event.target.value)} placeholder="https://disk.yandex.ru/... or cloud.mail.ru/..." required type="url" value={sourceUrl} /></div>
            <Button className="w-full md:w-auto" disabled={!year.active || availableMonths.length === 0} type="submit"><FolderPlus aria-hidden="true" />Link month</Button>
          </form>
          {availableMonths.length === 0 && <p className="mt-2 text-xs text-muted-foreground">All twelve months have been configured.</p>}
          {formError && <p className="mt-3 rounded-md bg-destructive/5 px-3 py-2 text-sm text-destructive" role="alert">{formError}</p>}
        </section>
        <div className="space-y-3">
          <div className="flex items-center justify-between"><h3 className="text-sm font-medium">Configured months</h3>{loading && <span className="text-xs text-muted-foreground">Loading…</span>}</div>
          {months.length === 0 && !loading && <div className="rounded-lg border border-dashed px-4 py-6 text-center text-sm text-muted-foreground">No months linked yet. Months without folders can stay empty.</div>}
          {months.map((month) => <MonthCard key={month.id} month={month} onChanged={reloadMonths} onReview={onReview} year={year} />)}
        </div>
      </CardContent>
      <ConfirmDialog confirmation={confirmation} onClose={() => setConfirmation(null)} />
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
    try {
      await api.createYear(feedId, value)
      setYearValue("")
      await reloadYears()
      toast.success(`${value} added`)
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Could not add year.")
    }
  }

  return (
    <div className="space-y-5">
      <Card>
        <CardHeader className="pb-4"><div className="flex items-start gap-3"><div className="flex size-9 items-center justify-center rounded-lg bg-primary/8 text-primary"><CalendarDays aria-hidden="true" className="size-4" /></div><div><CardTitle className="text-base">Years and source folders</CardTitle><CardDescription className="mt-1">Organize documents by year and link only months that have a folder.</CardDescription></div></div></CardHeader>
        <CardContent>
          <form className="flex flex-wrap items-end gap-3" onSubmit={(event) => void createYear(event)}>
          <div className="w-full max-w-[220px] space-y-1.5"><Label htmlFor={`new-year-${feedId}`}>Add a year</Label><Input id={`new-year-${feedId}`} onChange={(event) => setYearValue(event.target.value)} placeholder="e.g. 2026" required type="number" value={yearValue} /></div>
            <Button disabled={!active} type="submit"><FolderPlus aria-hidden="true" />Add year</Button>
          </form>
          {error && <p className="mt-3 rounded-md bg-destructive/5 px-3 py-2 text-sm text-destructive" role="alert">{error}</p>}
        </CardContent>
      </Card>
      <div className="space-y-4">
        {years.map((year) => <YearCard key={`${feedId}-${year.year}`} feedId={feedId} onChanged={reloadYears} onReview={onReviewMonth} year={year} />)}
        {years.length === 0 && !loading && <div className="rounded-xl border border-dashed bg-background px-6 py-10 text-center"><div className="mx-auto mb-3 flex size-10 items-center justify-center rounded-xl bg-muted text-muted-foreground"><CalendarDays aria-hidden="true" className="size-4" /></div><p className="text-sm font-medium">No years configured</p><p className="mt-1 text-sm text-muted-foreground">Add a year to start linking source folders.</p></div>}
        {loading && <div aria-label="Loading years" className="rounded-xl border bg-background px-6 py-8 text-center text-sm text-muted-foreground">Loading years…</div>}
      </div>
    </div>
  )
}
