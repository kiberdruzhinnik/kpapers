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

function response(body: unknown, status = 200) {
  return new Response(body === null ? null : JSON.stringify(body), {
    status,
    headers: body === null ? {} : { "Content-Type": "application/json" },
  })
}

function installDocumentsApi(failSave = false) {
  const fetchMock = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
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

function renderReview(initialSelection: { feedId: string; year: number; monthId: string } | null = { feedId: "feed-1", year: 2026, monthId: "month-1" }) {
  return render(<><Toaster /><PdfReview feeds={feeds} initialSelection={initialSelection} onBack={vi.fn()} /></>)
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
})
