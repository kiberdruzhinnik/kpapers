import { useEffect, useState } from "react"
import { ArrowLeft, ExternalLink, FileText, RefreshCw } from "lucide-react"
import { toast } from "sonner"
import { api, type Feed, type Month, type ReviewQueueDocument, type ReviewQueueMode, type SourceDocument, type Year } from "@/lib/api"
import { Field, FieldGroup, FieldLabel } from "@/components/ui/field"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"

const classificationLabel: Record<string, string> = {
  publish: "Suitable for publication",
  promotional: "Promotional",
  unknown: "Not classified",
}

type ReviewSelection = { feedId: string; year?: number; monthId?: string }
const monthNames = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"]

export function PdfReview({ feeds, initialSelection, initialQueueMode, onBack, onInvalidSelection, onQueueModeChange, onSelectionChange }: {
  feeds: Feed[]
  initialSelection: ReviewSelection | null
  initialQueueMode: ReviewQueueMode | null
  onBack: () => void
  onInvalidSelection?: () => void
  onQueueModeChange: (mode: ReviewQueueMode | null) => void
  onSelectionChange?: (selection: ReviewSelection | null) => void
}) {
  const [feedId, setFeedId] = useState(initialSelection?.feedId ?? feeds.find((feed) => feed.active)?.id ?? feeds[0]?.id ?? "")
  const [years, setYears] = useState<Year[]>([])
  const [year, setYear] = useState<number | null>(initialSelection?.year ?? null)
  const [months, setMonths] = useState<Month[]>([])
  const [monthId, setMonthId] = useState(initialSelection?.monthId ?? "")
  const [hierarchyLoading, setHierarchyLoading] = useState(Boolean(initialSelection))
  const [documents, setDocuments] = useState<SourceDocument[]>([])
  const [loading, setLoading] = useState(Boolean(initialSelection?.monthId))
  const [loadError, setLoadError] = useState<string | null>(null)
  const [queueResult, setQueueResult] = useState<{ mode: ReviewQueueMode; documents: ReviewQueueDocument[]; error?: string } | null>(null)
  const [savingIds, setSavingIds] = useState<Set<string>>(new Set())
  const [retryingIds, setRetryingIds] = useState<Set<string>>(new Set())
  const [saveErrors, setSaveErrors] = useState<Record<string, string>>({})

  useEffect(() => {
    let mounted = true
    if (!feedId || initialQueueMode) return
    api.years(feedId).then((items) => {
      if (!mounted) return
      if (initialSelection?.feedId === feedId && initialSelection.year !== undefined && !items.some((item) => item.year === initialSelection.year)) {
        toast.error("The requested review year is no longer available.")
        setYears(items)
        setYear(null)
        setMonthId("")
        setHierarchyLoading(false)
        onInvalidSelection?.()
        return
      }
      setYears(items)
      const nextYear = items.some((item) => item.year === year)
        ? year
        : initialSelection?.feedId === feedId && initialSelection.year !== undefined && items.some((item) => item.year === initialSelection.year)
          ? initialSelection.year
          : items[0]?.year ?? null
      setYear(nextYear)
      if (nextYear !== null && initialSelection?.year !== nextYear) onSelectionChange?.({ feedId, year: nextYear })
    }).catch((error: unknown) => {
      if (mounted) toast.error(error instanceof Error ? error.message : "Could not load feed years.")
    }).finally(() => { if (mounted) setHierarchyLoading(false) })
    return () => { mounted = false }
  }, [feedId, year, initialSelection, initialQueueMode, onInvalidSelection, onSelectionChange])

  useEffect(() => {
    let mounted = true
    if (!feedId || year === null || initialQueueMode) return
    api.months(feedId, year).then((items) => {
      if (!mounted) return
      if (initialSelection?.feedId === feedId && initialSelection.year === year && initialSelection.monthId !== undefined && !items.some((item) => item.id === initialSelection.monthId)) {
        toast.error("The requested review month is no longer available.")
        setMonths(items)
        setMonthId("")
        setHierarchyLoading(false)
        onInvalidSelection?.()
        return
      }
      setMonths(items)
      const nextMonthId = items.some((item) => item.id === monthId)
        ? monthId
        : initialSelection?.feedId === feedId && initialSelection.year === year && initialSelection.monthId !== undefined && items.some((item) => item.id === initialSelection.monthId)
          ? initialSelection.monthId
          : items[0]?.id ?? ""
      setMonthId(nextMonthId)
      setLoading(Boolean(nextMonthId))
      if (nextMonthId && initialSelection?.monthId !== nextMonthId) onSelectionChange?.({ feedId, year, monthId: nextMonthId })
    }).catch((error: unknown) => {
      if (mounted) toast.error(error instanceof Error ? error.message : "Could not load feed months.")
    }).finally(() => { if (mounted) setHierarchyLoading(false) })
    return () => { mounted = false }
  }, [feedId, year, initialSelection, initialQueueMode, monthId, onInvalidSelection, onSelectionChange])

  useEffect(() => {
    let mounted = true
    if (!monthId || initialQueueMode) return
    api.documents(monthId).then((items) => {
      if (mounted) setDocuments(items)
    }).catch((error: unknown) => {
      if (mounted) setLoadError(error instanceof Error ? error.message : "Could not load PDF documents.")
    }).finally(() => { if (mounted) setLoading(false) })
    return () => { mounted = false }
  }, [monthId, initialQueueMode])

  useEffect(() => {
    let mounted = true
    if (!initialQueueMode) return
    api.reviewQueue(initialQueueMode === "archived").then((items) => {
      if (mounted) setQueueResult({ mode: initialQueueMode, documents: items })
    }).catch((error: unknown) => {
      if (mounted) setQueueResult({ mode: initialQueueMode, documents: [], error: error instanceof Error ? error.message : "Could not load the review queue." })
    })
    return () => { mounted = false }
  }, [initialQueueMode])

  async function changeDecision(document: SourceDocument, value: string) {
    const override = value === "automatic" ? null : value as "include" | "exclude"
    setSavingIds((current) => new Set(current).add(document.id))
    setSaveErrors((current) => ({ ...current, [document.id]: "" }))
    try {
      await api.setPromotion(document.id, override)
      setDocuments((current) => current.map((item) => item.id === document.id ? { ...item, promotional_override: override } : item))
      setQueueResult((current) => current?.mode === initialQueueMode
        ? { ...current, documents: current.documents.map((item) => item.id === document.id ? { ...item, promotional_override: override } : item) }
        : current)
      toast.success("Publication decision saved", { description: document.filename })
    } catch (error) {
      const message = error instanceof Error ? error.message : "Could not save the publication decision."
      setSaveErrors((current) => ({ ...current, [document.id]: message }))
      toast.error("Could not save publication decision", { description: message })
    } finally {
      setSavingIds((current) => { const next = new Set(current); next.delete(document.id); return next })
    }
  }

  async function retryExtraction(document: SourceDocument) {
    if (!monthId && !initialQueueMode) return
    setRetryingIds((current) => new Set(current).add(document.id))
    try {
      await api.extractDocument(document.id, true)
      if (initialQueueMode) {
        setQueueResult({ mode: "needs-review", documents: await api.reviewQueue(false) })
      } else {
        setDocuments(await api.documents(monthId))
      }
      toast.success("PDF extraction complete", { description: document.filename })
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Could not extract this PDF.")
    } finally { setRetryingIds((current) => { const next = new Set(current); next.delete(document.id); return next }) }
  }

  const selectedFeed = feeds.find((feed) => feed.id === feedId)
  const selectedMonth = months.find((item) => item.id === monthId)
  const activeQueueResult = initialQueueMode && queueResult?.mode === initialQueueMode ? queueResult : null
  const queueLoading = Boolean(initialQueueMode && !activeQueueResult)
  const queueError = activeQueueResult?.error ?? null
  const queueDocuments = activeQueueResult?.documents ?? []

  return (
    <div className="flex flex-col gap-6">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div className="flex items-start gap-3">
          <Button aria-label="Back to feeds" onClick={onBack} size="icon" variant="outline"><ArrowLeft aria-hidden="true" /></Button>
          <div><h1>PDF review</h1><p className="mt-1 text-sm text-muted-foreground">Review extracted content and choose which documents appear in the feed.</p></div>
        </div>
        {!initialQueueMode && selectedFeed && year !== null && selectedMonth && <Badge className="h-auto max-w-full min-w-0 whitespace-normal break-words" variant="outline">{selectedFeed.display_name} · {year} · {monthNames[selectedMonth.month - 1] ?? `Month ${selectedMonth.month}`}</Badge>}
      </div>

      <div aria-label="PDF review filters" className="flex flex-wrap gap-2" role="group">
        <Button aria-pressed={initialQueueMode === "needs-review"} onClick={() => onQueueModeChange("needs-review")} variant={initialQueueMode === "needs-review" ? "default" : "outline"}>Needs review</Button>
        <Button aria-pressed={initialQueueMode === "archived"} onClick={() => onQueueModeChange("archived")} variant={initialQueueMode === "archived" ? "default" : "outline"}>Archived</Button>
        <Button aria-pressed={!initialQueueMode} onClick={() => onQueueModeChange(null)} variant={!initialQueueMode ? "default" : "outline"}>Browse by source</Button>
      </div>

      {initialQueueMode ? <Card>
        <CardHeader>
          <CardTitle>{initialQueueMode === "archived" ? "Archived documents needing attention" : "Documents needing attention"}</CardTitle>
          <CardDescription>{initialQueueMode === "archived" ? "These documents belong to an archived feed, year, or month." : "Documents across active feeds, years, and months that need your decision."}</CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-3">
          {queueLoading && <p className="py-8 text-center text-sm text-muted-foreground">Loading documents…</p>}
          {queueError && <Alert variant="destructive"><AlertDescription>{queueError}</AlertDescription></Alert>}
          {!queueLoading && !queueError && queueDocuments.length === 0 && <Empty><EmptyHeader><EmptyMedia variant="icon"><FileText aria-hidden="true" /></EmptyMedia><EmptyTitle>No documents need attention</EmptyTitle><EmptyDescription>This review queue is empty.</EmptyDescription></EmptyHeader></Empty>}
          {!queueLoading && !queueError && queueDocuments.map((document) => renderDocument(document, initialQueueMode === "archived"))}
        </CardContent>
      </Card> : <>
        <Card>
          <CardHeader><CardTitle>Choose a source</CardTitle><CardDescription>Browse discovered documents by feed, year, and month.</CardDescription></CardHeader>
          <CardContent><FieldGroup className="grid gap-4 sm:grid-cols-3">
            <Field><FieldLabel htmlFor="review-feed">Feed</FieldLabel>
              <select id="review-feed" aria-label="Feed" className="h-9 min-w-0 w-full rounded-md border border-input bg-background px-3 text-sm font-normal" onChange={(event) => { const nextFeedId = event.target.value; setFeedId(nextFeedId); setYear(null); setMonthId(""); setDocuments([]); setLoadError(null); setHierarchyLoading(true); setLoading(false); onSelectionChange?.(nextFeedId ? { feedId: nextFeedId } : null) }} value={feedId}>
                <option value="">Choose a feed</option>
                {feeds.map((feed) => <option key={feed.id} value={feed.id}>{feed.display_name}</option>)}
              </select>
            </Field>
            <Field><FieldLabel htmlFor="review-year">Year</FieldLabel>
              <select id="review-year" aria-label="Year" className="h-9 min-w-0 w-full rounded-md border border-input bg-background px-3 text-sm font-normal" disabled={!feedId || hierarchyLoading} onChange={(event) => { const nextYear = event.target.value ? Number(event.target.value) : undefined; setYear(nextYear ?? null); setMonthId(""); setDocuments([]); setLoadError(null); setHierarchyLoading(true); setLoading(false); onSelectionChange?.(nextYear === undefined ? { feedId } : { feedId, year: nextYear }) }} value={year ?? ""}>
                <option value="">{hierarchyLoading ? "Loading years…" : "Choose a year"}</option>
                {years.map((item) => <option key={item.year} value={item.year}>{item.year}</option>)}
              </select>
            </Field>
            <Field><FieldLabel htmlFor="review-month">Month</FieldLabel>
              <select id="review-month" aria-label="Month" className="h-9 min-w-0 w-full rounded-md border border-input bg-background px-3 text-sm font-normal" disabled={year === null || hierarchyLoading} onChange={(event) => { const nextMonthId = event.target.value; setMonthId(nextMonthId); setDocuments([]); setLoadError(null); setLoading(Boolean(nextMonthId)); onSelectionChange?.(nextMonthId ? { feedId, year: year!, monthId: nextMonthId } : { feedId, year: year! }) }} value={monthId}>
                <option value="">{hierarchyLoading ? "Loading months…" : "Choose a month"}</option>
                {months.map((item) => <option key={item.id} value={item.id}>{monthNames[item.month - 1] ?? `Month ${item.month}`}</option>)}
              </select>
            </Field>
          </FieldGroup></CardContent>
        </Card>
        {!monthId ? <Card><CardContent className="py-12 text-center"><FileText className="mx-auto size-8 text-muted-foreground" /><p className="mt-3 text-sm font-medium">{hierarchyLoading ? "Loading source folders…" : "Choose a month to review its PDFs"}</p></CardContent></Card>
          : <Card>
            <CardHeader><CardTitle>Discovered PDFs</CardTitle><CardDescription>Publication overrides take precedence over automatic promotion classification.</CardDescription></CardHeader>
            <CardContent className="flex flex-col gap-3">
              {loading && <p className="py-8 text-center text-sm text-muted-foreground">Loading documents…</p>}
              {loadError && <Alert variant="destructive"><AlertDescription>{loadError}</AlertDescription></Alert>}
              {!loading && !loadError && documents.length === 0 && <Empty><EmptyHeader><EmptyMedia variant="icon"><FileText aria-hidden="true" /></EmptyMedia><EmptyTitle>No PDFs found</EmptyTitle><EmptyDescription>Scan this folder from feed management to discover documents.</EmptyDescription></EmptyHeader></Empty>}
              {!loading && !loadError && documents.map((document) => renderDocument(document, false))}
            </CardContent>
          </Card>}
      </>}
    </div>
  )

  function renderDocument(document: SourceDocument | ReviewQueueDocument, archived: boolean) {
    const classification = document.promotional_classification
    const status = document.extraction_status.toLowerCase()
    const queueDocument = "feed_name" in document ? document : null
    return <article className="grid min-w-0 gap-4 rounded-xl border p-4 lg:grid-cols-[minmax(0,1fr)_minmax(200px,0.55fr)] lg:items-center" key={document.id}>
      <div className="min-w-0">
        <div className="flex min-w-0 items-start gap-3"><div className="mt-0.5 flex size-9 shrink-0 items-center justify-center rounded-lg bg-muted text-muted-foreground"><FileText aria-hidden="true" className="size-4" /></div><div className="min-w-0 flex-1"><h2 className="break-words text-sm font-medium">{document.filename}</h2><div className="mt-2 flex flex-wrap items-center gap-2"><Badge variant={status === "success" ? "secondary" : status === "failure" ? "destructive" : "outline"}>Extraction {document.extraction_status}</Badge><Badge variant={classification === "publish" ? "secondary" : "outline"}>{classificationLabel[classification] ?? classification}</Badge></div></div></div>
        {queueDocument && <p className="ml-12 mt-2 text-xs text-muted-foreground">{queueDocument.feed_name} · {queueDocument.year} · {monthNames[queueDocument.month - 1] ?? `Month ${queueDocument.month}`}</p>}
        {document.last_error && <p className="ml-12 mt-2 break-words text-xs text-destructive">{document.last_error}</p>}
        {!archived && <a className="ml-12 mt-2 inline-flex items-center gap-1 text-xs text-primary underline-offset-4 hover:underline" href={api.documentSourceUrl(document.id)} rel="noreferrer" target="_blank">View source PDF <ExternalLink aria-hidden="true" className="size-3" /></a>}
        {status !== "success" && !archived && <Button className="ml-9 mt-2" disabled={retryingIds.has(document.id)} onClick={() => void retryExtraction(document)} size="sm" variant="ghost"><RefreshCw aria-hidden="true" />{retryingIds.has(document.id) ? "Extracting…" : "Retry extraction"}</Button>}
        {archived && <p className="ml-12 mt-2 text-xs text-muted-foreground">Restore this archived source to open its PDF or retry extraction.</p>}
      </div>
      <div className="flex flex-col gap-1.5"><label className="text-sm font-medium" htmlFor={`decision-${document.id}`}>Publication decision</label><select aria-label={`Publication decision for ${document.filename}`} className="h-9 min-w-0 w-full rounded-md border border-input bg-background px-3 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" disabled={savingIds.has(document.id)} id={`decision-${document.id}`} onChange={(event) => void changeDecision(document, event.target.value)} value={document.promotional_override ?? "automatic"}><option value="automatic">Automatic · {classificationLabel[document.promotional_classification] ?? document.promotional_classification}</option><option value="include">Include in feed</option><option value="exclude">Exclude from feed</option></select>{status !== "success" && <p className="text-xs text-muted-foreground">Successful extraction is required before this PDF can appear in the feed.</p>}{savingIds.has(document.id) && <p role="status" className="text-xs text-muted-foreground">Saving decision…</p>}{saveErrors[document.id] && <p className="text-xs text-destructive" role="alert">{saveErrors[document.id]}</p>}</div>
    </article>
  }
}
