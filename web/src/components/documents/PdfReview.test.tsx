import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it, vi } from "vitest"
import { PdfReview } from "@/components/documents/PdfReview"
import { Toaster } from "@/components/ui/sonner"

const document = {
  id: "doc-1", filename: "research-paper.pdf", source_url: "https://example.test/research-paper.pdf",
  promotional_classification: "publish", promotional_override: null, extraction_status: "success", last_error: null,
}
const feeds = [
  { id: "feed-1", display_name: "Ada Lovelace", description: "", active: true, refresh_interval_seconds: null },
  { id: "feed-2", display_name: "Grace Hopper", description: "", active: true, refresh_interval_seconds: null },
]
const selectedMonth = { id: "month-1", feed_id: "feed-1", year: 2026, month: 11, provider: "fixture", source_url: "fixture://current-month/", active: true, last_attempt_at: null, next_refresh_at: null, last_result: null, last_error: null }
const secondDocument = { ...document, id: "doc-2", filename: "compiler-notes.pdf" }
const queuedDocument = {
  ...document,
  id: "queued-doc",
  filename: "queued-paper.pdf",
  extraction_status: "failure",
  feed_id: "feed-2",
  feed_name: "Grace Hopper",
  year: 2025,
  month: 12,
  month_id: "month-2",
  archived: false,
}

function response(body: unknown, status = 200) {
  return new Response(body === null ? null : JSON.stringify(body), {
    status,
    headers: body === null ? {} : { "Content-Type": "application/json" },
  })
}

function installDocumentsApi(failSave = false, queueDocuments: unknown[] = [queuedDocument]) {
  const fetchMock = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
    if (String(input) === "/api/management/review-queue?archived=false" || String(input) === "/api/management/review-queue?archived=true") return response(queueDocuments)
    if (String(input) === "/api/management/feeds/feed-1/years") return response([{ feed_id: "feed-1", year: 2026, active: true }])
    if (String(input) === "/api/management/feeds/feed-2/years") return response([{ feed_id: "feed-2", year: 2025, active: true }])
    if (String(input) === "/api/management/feeds/feed-1/years/2026/months") return response([selectedMonth])
    if (String(input) === "/api/management/feeds/feed-2/years/2025/months") return response([{ ...selectedMonth, id: "month-2", feed_id: "feed-2", year: 2025, month: 12 }])
    if (String(input) === "/api/management/months/month-1/documents") return response([document])
    if (String(input) === "/api/management/months/month-2/documents") return response([secondDocument])
    if (String(input) === "/api/management/documents/doc-1/promotion" && init?.method === "PUT") {
      return failSave ? response(null, 500) : response(null, 204)
    }
    return response(null, 404)
  })
  vi.stubGlobal("fetch", fetchMock)
  return fetchMock
}

function renderReview(initialSelection: { feedId: string; year: number; monthId: string } | null = { feedId: "feed-1", year: 2026, monthId: "month-1" }, initialQueueMode: "needs-review" | "archived" | null = null) {
  return render(<><Toaster /><PdfReview feeds={feeds} initialSelection={initialSelection} initialQueueMode={initialQueueMode} onBack={vi.fn()} onQueueModeChange={vi.fn()} /></>)
}

describe("PDF review", () => {
  it("saves automatic, include, and exclude publication choices", async () => {
    const fetchMock = installDocumentsApi()
    const user = userEvent.setup()
    renderReview()

    const decision = await screen.findByRole("combobox", { name: "Publication decision for research-paper.pdf" })
    expect(screen.getByRole("combobox", { name: "Feed" })).toHaveValue("feed-1")
    expect(screen.getByRole("combobox", { name: "Year" })).toHaveValue("2026")
    expect(screen.getByRole("combobox", { name: "Month" })).toHaveValue("month-1")
    expect(screen.getByRole("link", { name: "View source PDF" })).toHaveAttribute("href", "/api/management/documents/doc-1/source")
    await user.selectOptions(decision, "include")
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith("/api/management/documents/doc-1/promotion", expect.objectContaining({ method: "PUT", body: JSON.stringify({ override_value: "include" }) })))
    await user.selectOptions(decision, "exclude")
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith("/api/management/documents/doc-1/promotion", expect.objectContaining({ method: "PUT", body: JSON.stringify({ override_value: "exclude" }) })))
    await user.selectOptions(decision, "automatic")
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith("/api/management/documents/doc-1/promotion", expect.objectContaining({ method: "PUT", body: JSON.stringify({ override_value: null }) })))
    expect((await screen.findAllByText("Publication decision saved")).length).toBeGreaterThan(0)
  })

  it("keeps the existing choice and shows feedback when a save fails", async () => {
    installDocumentsApi(true)
    const user = userEvent.setup()
    renderReview()

    const decision = await screen.findByRole("combobox", { name: "Publication decision for research-paper.pdf" })
    await user.selectOptions(decision, "exclude")
    expect(await screen.findByRole("alert")).toHaveTextContent("Kpapers could not complete that request. Try again shortly.")
    expect(decision).toHaveValue("automatic")
  })

  it("shows queue loading and per-document save-pending feedback", async () => {
    const fetchMock = installDocumentsApi()
    const original = fetchMock.getMockImplementation()!
    let resolveQueue!: (result: Response) => void
    let resolveSave!: (result: Response) => void
    fetchMock.mockImplementation((input, init) => {
      if (String(input) === "/api/management/review-queue?archived=false") return new Promise((resolve) => { resolveQueue = resolve })
      if (String(input) === "/api/management/documents/doc-1/promotion" && init?.method === "PUT") return new Promise((resolve) => { resolveSave = resolve })
      return original(input, init)
    })
    const user = userEvent.setup()
    renderReview(null, "needs-review")

    expect(await screen.findByText("Loading documents…")).toBeInTheDocument()
    resolveQueue(response([{ ...document, feed_name: "Ada Lovelace", year: 2026, month: 11 }]))
    const decision = await screen.findByRole("combobox", { name: "Publication decision for research-paper.pdf" })
    await user.selectOptions(decision, "exclude")
    expect(await screen.findByText("Saving decision…")).toBeInTheDocument()
    expect(decision).toBeDisabled()
    resolveSave(response(null, 204))
    await waitFor(() => expect(decision).toHaveValue("exclude"))
  })

  it("reports review queue load errors and keeps archived source actions unavailable", async () => {
    const fetchMock = installDocumentsApi(false, [{ ...queuedDocument, archived: true }])
    const original = fetchMock.getMockImplementation()!
    fetchMock.mockImplementation((input, init) => {
      if (String(input) === "/api/management/review-queue?archived=true") return Promise.resolve(response(null, 500))
      return original(input, init)
    })
    const { unmount } = renderReview(null, "archived")
    expect(await screen.findByRole("alert")).toHaveTextContent("Kpapers could not complete that request. Try again shortly.")
    unmount()

    installDocumentsApi(false, [{ ...queuedDocument, archived: true }])
    renderReview(null, "archived")
    expect(await screen.findByRole("heading", { name: "queued-paper.pdf" })).toBeInTheDocument()
    expect(screen.getByText(/Restore this archived source/)).toBeInTheDocument()
    expect(screen.queryByRole("link", { name: "View source PDF" })).not.toBeInTheDocument()
    expect(screen.queryByRole("button", { name: "Retry extraction" })).not.toBeInTheDocument()
  })

  it("navigates by feed, year, and month and loads the selected month's documents", async () => {
    installDocumentsApi()
    const user = userEvent.setup()
    renderReview(null)

    await screen.findByRole("combobox", { name: "Publication decision for research-paper.pdf" })
    await user.selectOptions(screen.getByRole("combobox", { name: "Feed" }), "feed-2")

    expect(await screen.findByRole("combobox", { name: "Publication decision for compiler-notes.pdf" })).toBeInTheDocument()
    expect(screen.getByRole("combobox", { name: "Year" })).toHaveValue("2025")
    expect(screen.getByRole("combobox", { name: "Month" })).toHaveValue("month-2")
    expect(screen.getByText("Grace Hopper · 2025 · December")).toBeInTheDocument()
  })

  it("keeps documents visible when they load before the selected month is validated", async () => {
    const fetchMock = installDocumentsApi()
    const original = fetchMock.getMockImplementation()!
    let resolveMonths!: (result: Response) => void
    fetchMock.mockImplementation((input, init) => {
      if (String(input) === "/api/management/feeds/feed-1/years/2026/months") {
        return new Promise((resolve) => { resolveMonths = resolve })
      }
      return original(input, init)
    })

    renderReview()
    await waitFor(() => expect(fetchMock.mock.calls.some(([input]) => String(input) === "/api/management/months/month-1/documents")).toBe(true))
    await screen.findByRole("heading", { name: "research-paper.pdf" })
    resolveMonths(response([selectedMonth]))

    expect(await screen.findByRole("heading", { name: "research-paper.pdf" })).toBeInTheDocument()
    expect(screen.queryByText("Loading documents…")).not.toBeInTheDocument()
  })
})

describe("classification and extraction presentation", () => {
  it("keeps automatic classification visible independently of the saved override", async () => {
    installDocumentsApi()
    const user = userEvent.setup()
    renderReview()
    const decision = await screen.findByRole("combobox", { name: "Publication decision for research-paper.pdf" })
    await user.selectOptions(decision, "exclude")
    await waitFor(() => expect(decision).toHaveValue("exclude"))
    expect(screen.getByText("Suitable for publication", { selector: "[data-slot=badge]" })).toBeInTheDocument()
  })

  it("explains extraction eligibility and retries a failed PDF", async () => {
    const fetchMock = installDocumentsApi()
    const original = fetchMock.getMockImplementation()!
    let extracted = false
    fetchMock.mockImplementation(async (input, init) => {
      if (String(input) === "/api/management/months/month-1/documents") return response([{ ...document, extraction_status: extracted ? "success" : "failure", last_error: extracted ? null : "Extraction failed" }])
      if (String(input) === "/api/management/documents/doc-1/extract?force=true" && init?.method === "POST") { extracted = true; return response(null, 204) }
      return original(input, init)
    })
    const user = userEvent.setup()
    renderReview()
    await screen.findByText("Extraction failure")
    expect(screen.getByText("Extraction failed")).toBeInTheDocument()
    expect(screen.getByText(/Successful extraction is required/)).toBeInTheDocument()
    await user.click(screen.getByRole("button", { name: "Retry extraction" }))
    await screen.findByText("Extraction success")
    expect(screen.queryByText("Extraction failed")).not.toBeInTheDocument()
    expect(screen.queryByText(/Successful extraction is required/)).not.toBeInTheDocument()
  })

  it("loads the global active queue with source context and saves a publication decision", async () => {
    const fetchMock = installDocumentsApi()
    const user = userEvent.setup()
    renderReview(null, "needs-review")

    expect(await screen.findByRole("heading", { name: "Documents needing attention" })).toBeInTheDocument()
    expect(await screen.findByRole("heading", { name: "queued-paper.pdf" })).toBeInTheDocument()
    expect(screen.getByText("Grace Hopper · 2025 · December")).toBeInTheDocument()
    expect(screen.getByRole("link", { name: "View source PDF" })).toHaveAttribute("href", "/api/management/documents/queued-doc/source")
    const decision = screen.getByRole("combobox", { name: "Publication decision for queued-paper.pdf" })
    await user.selectOptions(decision, "include")
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith("/api/management/documents/queued-doc/promotion", expect.objectContaining({ method: "PUT", body: JSON.stringify({ override_value: "include" }) })))
  })

  it("shows a clear state for an empty archived queue", async () => {
    installDocumentsApi(false, [])
    renderReview(null, "archived")

    expect(await screen.findByRole("heading", { name: "Archived documents needing attention" })).toBeInTheDocument()
    expect(await screen.findByText("No documents need attention")).toBeInTheDocument()
    expect(screen.getByText("This review queue is empty.")).toBeInTheDocument()
  })
})
