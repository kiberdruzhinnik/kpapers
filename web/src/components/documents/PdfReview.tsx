import { useEffect, useState } from "react"
import { ArrowLeft, ExternalLink, FileText, RefreshCw } from "lucide-react"
import { toast } from "sonner"
import { api, type Feed, type Month, type SourceDocument, type Year } from "@/lib/api"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"

const classificationLabel: Record<string, string> = {
  publish: "Suitable for publication",
  promotional: "Promotional",
  unknown: "Not classified",
}

type ReviewSelection = { feedId: string; year: number; monthId: string }
const monthNames = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"]

export function PdfReview({ feeds, initialSelection, onBack }: { feeds: Feed[]; initialSelection: ReviewSelection | null; onBack: () => void }) {
  const [feedId, setFeedId] = useState(initialSelection?.feedId ?? feeds.find((feed) => feed.active)?.id ?? feeds[0]?.id ?? "")
  const [years, setYears] = useState<Year[]>([])
  const [year, setYear] = useState<number | null>(initialSelection?.year ?? null)
  const [months, setMonths] = useState<Month[]>([])
  const [monthId, setMonthId] = useState(initialSelection?.monthId ?? "")
  const [hierarchyLoading, setHierarchyLoading] = useState(Boolean(initialSelection))
  const [documents, setDocuments] = useState<SourceDocument[]>([])
  const [loading, setLoading] = useState(Boolean(initialSelection?.monthId))
  const [loadError, setLoadError] = useState<string | null>(null)
  const [savingId, setSavingId] = useState<string | null>(null)
  const [saveError, setSaveError] = useState<{ id: string; message: string } | null>(null)

  useEffect(() => {
    let mounted = true
    if (!feedId) {
      return
    }
    api.years(feedId).then((items) => {
      if (!mounted) return
      setYears(items)
      setYear((current) => items.some((item) => item.year === current)
        ? current
        : initialSelection?.feedId === feedId && items.some((item) => item.year === initialSelection.year)
          ? initialSelection.year
          : items[0]?.year ?? null)
    }).catch((error: unknown) => {
      if (mounted) toast.error(error instanceof Error ? error.message : "Could not load feed years.")
    }).finally(() => { if (mounted) setHierarchyLoading(false) })
    return () => { mounted = false }
  }, [feedId, initialSelection])

  useEffect(() => {
    let mounted = true
    if (!feedId || year === null) {
      return
    }
    api.months(feedId, year).then((items) => {
      if (!mounted) return
      setMonths(items)
      setMonthId((current) => items.some((item) => item.id === current)
        ? current
        : initialSelection?.feedId === feedId && initialSelection.year === year && items.some((item) => item.id === initialSelection.monthId)
          ? initialSelection.monthId
          : items[0]?.id ?? "")
    }).catch((error: unknown) => {
      if (mounted) toast.error(error instanceof Error ? error.message : "Could not load feed months.")
    }).finally(() => { if (mounted) setHierarchyLoading(false) })
    return () => { mounted = false }
  }, [feedId, year, initialSelection])

  useEffect(() => {
    let mounted = true
    if (!monthId) {
      return
    }
    api.documents(monthId).then((items) => {
      if (mounted) setDocuments(items)
    }).catch((error: unknown) => {
      if (mounted) setLoadError(error instanceof Error ? error.message : "Could not load PDF documents.")
    }).finally(() => { if (mounted) setLoading(false) })
    return () => { mounted = false }
  }, [monthId])

  async function changeDecision(document: SourceDocument, value: string) {
    const override = value === "automatic" ? null : value as "include" | "exclude"
    setSavingId(document.id)
    setSaveError(null)
    try {
      await api.setPromotion(document.id, override)
      setDocuments((current) => current.map((item) => item.id === document.id ? { ...item, promotional_override: override } : item))
      toast.success("Publication decision saved", { description: document.filename })
    } catch (error) {
      const message = error instanceof Error ? error.message : "Could not save the publication decision."
      setSaveError({ id: document.id, message })
      toast.error("Could not save publication decision", { description: message })
    } finally {
      setSavingId(null)
    }
  }

  async function retryExtraction(document: SourceDocument) {
    if (!monthId) return
    try {
      await api.extractDocument(document.id, true)
      const updated = await api.documents(monthId)
      setDocuments(updated)
      toast.success("PDF extraction complete", { description: document.filename })
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Could not extract this PDF.")
    }
  }

  const selectedFeed = feeds.find((feed) => feed.id === feedId)
  const selectedMonth = months.find((item) => item.id === monthId)

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div className="flex items-start gap-3">
          <Button aria-label="Back to feeds" onClick={onBack} size="icon" variant="outline"><ArrowLeft aria-hidden="true" /></Button>
          <div><p className="mb-1 text-sm text-muted-foreground">Publication controls</p><h1 className="text-2xl font-semibold tracking-tight">PDF review</h1><p className="mt-1 text-sm text-muted-foreground">Review extracted content and choose which documents appear in the feed.</p></div>
        </div>
        {selectedFeed && year !== null && selectedMonth && <Badge variant="outline">{selectedFeed.display_name} · {year} · {monthNames[selectedMonth.month - 1] ?? `Month ${selectedMonth.month}`}</Badge>}
      </div>

      <Card>
        <CardHeader><CardTitle>Choose a source</CardTitle><CardDescription>Browse discovered documents by feed, year, and month.</CardDescription></CardHeader>
        <CardContent className="grid gap-4 sm:grid-cols-3">
          <label className="space-y-1.5 text-xs font-medium">Feed
            <select aria-label="Feed" className="h-9 w-full rounded-md border border-input bg-background px-3 text-sm font-normal" onChange={(event) => { setFeedId(event.target.value); setYear(null); setMonthId(""); setDocuments([]); setLoadError(null); setHierarchyLoading(true); setLoading(false) }} value={feedId}>
              <option value="">Choose a feed</option>
              {feeds.map((feed) => <option key={feed.id} value={feed.id}>{feed.display_name}</option>)}
            </select>
          </label>
          <label className="space-y-1.5 text-xs font-medium">Year
            <select aria-label="Year" className="h-9 w-full rounded-md border border-input bg-background px-3 text-sm font-normal" disabled={!feedId || hierarchyLoading} onChange={(event) => { setYear(event.target.value ? Number(event.target.value) : null); setMonthId(""); setDocuments([]); setLoadError(null); setHierarchyLoading(true); setLoading(false) }} value={year ?? ""}>
              <option value="">{hierarchyLoading ? "Loading years…" : "Choose a year"}</option>
              {years.map((item) => <option key={item.year} value={item.year}>{item.year}</option>)}
            </select>
          </label>
          <label className="space-y-1.5 text-xs font-medium">Month
            <select aria-label="Month" className="h-9 w-full rounded-md border border-input bg-background px-3 text-sm font-normal" disabled={year === null || hierarchyLoading} onChange={(event) => { setMonthId(event.target.value); setDocuments([]); setLoadError(null); setLoading(Boolean(event.target.value)) }} value={monthId}>
              <option value="">{hierarchyLoading ? "Loading months…" : "Choose a month"}</option>
              {months.map((item) => <option key={item.id} value={item.id}>{monthNames[item.month - 1] ?? `Month ${item.month}`}</option>)}
            </select>
          </label>
        </CardContent>
      </Card>

      {!monthId ? <Card><CardContent className="py-12 text-center"><FileText className="mx-auto size-8 text-muted-foreground" /><p className="mt-3 text-sm font-medium">{hierarchyLoading ? "Loading source folders…" : "Choose a month to review its PDFs"}</p></CardContent></Card>
        : <Card>
          <CardHeader><CardTitle>Discovered PDFs</CardTitle><CardDescription>Publication overrides take precedence over automatic promotion classification.</CardDescription></CardHeader>
          <CardContent className="space-y-3">
            {loading && <p className="py-8 text-center text-sm text-muted-foreground">Loading documents…</p>}
            {loadError && <p className="rounded-md bg-destructive/5 px-3 py-2 text-sm text-destructive" role="alert">{loadError}</p>}
            {!loading && !loadError && documents.length === 0 && <div className="rounded-lg border border-dashed px-5 py-10 text-center"><FileText className="mx-auto size-7 text-muted-foreground" /><p className="mt-3 text-sm font-medium">No PDFs found</p><p className="mt-1 text-sm text-muted-foreground">Scan this folder from feed management to discover documents.</p></div>}
            {documents.map((document) => {
              const effective = document.promotional_override ?? document.promotional_classification
              const status = document.extraction_status.toLowerCase()
              return <article className="grid min-w-0 gap-4 rounded-xl border p-4 sm:grid-cols-[minmax(0,1fr)_minmax(200px,0.55fr)] sm:items-center" key={document.id}>
                <div className="min-w-0">
                  <div className="flex min-w-0 items-start gap-3"><div className="mt-0.5 flex size-9 shrink-0 items-center justify-center rounded-lg bg-muted text-muted-foreground"><FileText aria-hidden="true" className="size-4" /></div><div className="min-w-0 flex-1"><h2 className="break-words text-sm font-medium">{document.filename}</h2><div className="mt-2 flex flex-wrap items-center gap-2"><Badge variant={status === "success" ? "secondary" : status === "failure" ? "destructive" : "outline"}>Extraction {document.extraction_status}</Badge><Badge variant={effective === "publish" || effective === "include" ? "secondary" : "outline"}>{classificationLabel[effective] ?? effective}</Badge></div></div></div>
                  {document.last_error && <p className="ml-12 mt-2 break-words text-xs text-destructive">{document.last_error}</p>}
                  <a className="ml-12 mt-2 inline-flex items-center gap-1 text-xs text-primary underline-offset-4 hover:underline" href={api.documentSourceUrl(document.id)} rel="noreferrer" target="_blank">View source PDF <ExternalLink aria-hidden="true" className="size-3" /></a>
                  {status !== "success" && <Button className="ml-9 mt-2" onClick={() => void retryExtraction(document)} size="sm" variant="ghost"><RefreshCw aria-hidden="true" />Retry extraction</Button>}
                </div>
                <div className="space-y-1.5"><label className="text-xs font-medium" htmlFor={`decision-${document.id}`}>Publication decision</label><select aria-label={`Publication decision for ${document.filename}`} className="h-9 w-full rounded-md border border-input bg-background px-3 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" disabled={savingId === document.id} id={`decision-${document.id}`} onChange={(event) => void changeDecision(document, event.target.value)} value={document.promotional_override ?? "automatic"}><option value="automatic">Automatic · {classificationLabel[document.promotional_classification] ?? document.promotional_classification}</option><option value="include">Include in feed</option><option value="exclude">Exclude from feed</option></select>{savingId === document.id && <p className="text-xs text-muted-foreground">Saving decision…</p>}{saveError?.id === document.id && <p className="text-xs text-destructive" role="alert">{saveError.message}</p>}</div>
              </article>
            })}
          </CardContent>
        </Card>}
    </div>
  )
}
