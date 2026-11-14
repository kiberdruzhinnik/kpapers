import { render, screen, waitFor, within } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { beforeEach, describe, expect, it, vi } from "vitest"
import App from "./App"

const sampleFeed = {
  id: "feed-1",
  display_name: "Ada Lovelace",
  description: "Research notes",
  active: true,
  refresh_interval_seconds: null,
  extraction_engine: null,
}
const secondFeed = { ...sampleFeed, id: "feed-2", display_name: "Grace Hopper", description: "Compiler notes" }
const sampleMonth = {
  id: "month-1", feed_id: "feed-1", year: 2026, month: 11, provider: "fixture", source_url: "fixture://source/",
  active: true, last_attempt_at: null, next_refresh_at: null, last_result: null, last_error: null,
}
const secondMonth = { ...sampleMonth, id: "month-2", feed_id: "feed-2", year: 2025, month: 12 }

function response(body: unknown, status = 200) {
  return new Response(body === null ? null : JSON.stringify(body), {
    status,
    headers: body === null ? {} : { "Content-Type": "application/json" },
  })
}

function installApi(initiallySignedIn = false, overviewResponse: () => Promise<Response> = async () => response({ source_folder_count: 3, needs_review_count: 2 })) {
  window.history.replaceState({}, "", "/")
  let signedIn = initiallySignedIn
  const fetchMock = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
    const path = String(input)
    if (path === "/api/management/session" && init?.method === "DELETE") {
      signedIn = false
      return response(null, 204)
    }
    if (path === "/api/management/session") return signedIn ? response({ status: "authenticated" }) : response(null, 401)
    if (path === "/api/management/login") {
      const credentials = JSON.parse(String(init?.body)) as { username: string; password: string }
      if (credentials.username !== "operator" || credentials.password !== "correct horse") return response(null, 401)
      signedIn = true
      return response({ authenticated: true })
    }
    if (path === "/api/management/feeds") return signedIn ? response([sampleFeed, secondFeed]) : response(null, 401)
    if (path === "/api/management/overview") return signedIn ? overviewResponse() : response(null, 401)
    if (path === "/api/management/feeds/feed-1/years") return signedIn ? response([{ feed_id: "feed-1", year: 2026, active: true }]) : response(null, 401)
    if (path === "/api/management/feeds/feed-2/years") return signedIn ? response([{ feed_id: "feed-2", year: 2025, active: true }]) : response(null, 401)
    if (path === "/api/management/feeds/feed-1/years/2026/months") return signedIn ? response([sampleMonth]) : response(null, 401)
    if (path === "/api/management/feeds/feed-2/years/2025/months") return signedIn ? response([secondMonth]) : response(null, 401)
    if (path === "/api/management/months/month-1/documents") return signedIn ? response([]) : response(null, 401)
    if (path === "/api/management/months/month-2/documents") return signedIn ? response([]) : response(null, 401)
    return response(null, 404)
  })
  vi.stubGlobal("fetch", fetchMock)
  return fetchMock
}

describe("operator session and dashboard shell", () => {
  beforeEach(() => localStorage.clear())
  it("restores a signed-in session and displays the workspace", async () => {
    installApi(true)

    render(<App />)

    expect(await screen.findByRole("heading", { name: "Overview" })).toBeInTheDocument()
    expect(screen.getByText("Active feeds")).toBeInTheDocument()
    expect(screen.getByRole("button", { name: "Create feed" })).toBeInTheDocument()
    await waitFor(() => {
      expect(screen.getByTestId("source-folder-count")).toHaveTextContent("3")
      expect(screen.getByTestId("needs-review-count")).toHaveTextContent("2")
    })
    expect(screen.getByText("Publishing workflow")).toBeInTheDocument()
    expect(screen.getByText("2 documents need attention")).toBeInTheDocument()
    expect(screen.queryByText("Set up your first author feed in a few steps.")).not.toBeInTheDocument()
    expect(screen.queryByText("Refresh scheduler")).not.toBeInTheDocument()
    expect(screen.queryByText("Kpapers operator desk")).not.toBeInTheDocument()
    expect(screen.queryByText("Feeds update on your schedule")).not.toBeInTheDocument()
  })

  it("keeps management private and supports keyboard-accessible sign-in feedback", async () => {
    installApi(false)
    const user = userEvent.setup()

    render(<App />)

    const username = await screen.findByRole("textbox", { name: "Username" })
    expect(username).toHaveFocus()
    await user.tab()
    expect(screen.getByLabelText("Password")).toHaveFocus()
    await user.tab()
    expect(screen.getByRole("button", { name: "Sign in" })).toHaveFocus()
    expect(screen.queryByRole("navigation", { name: "Primary navigation" })).not.toBeInTheDocument()

    await user.type(username, "operator")
    await user.type(screen.getByLabelText("Password"), "wrong password")
    await user.click(screen.getByRole("button", { name: "Sign in" }))
    expect(await screen.findByRole("alert")).toHaveTextContent("The username or password is incorrect.")

    await user.clear(screen.getByLabelText("Password"))
    await user.type(screen.getByLabelText("Password"), "correct horse")
    await user.click(screen.getByRole("button", { name: "Sign in" }))
    expect(await screen.findByRole("heading", { name: "Overview" })).toBeInTheDocument()
  })

  it("navigates between dashboard sections and signs out", async () => {
    const fetchMock = installApi(true)
    const user = userEvent.setup()

    render(<App />)

    await screen.findByRole("heading", { name: "Overview" })
    const nav = screen.getByRole("navigation", { name: "Primary navigation" })
    await user.click(within(nav).getByRole("button", { name: /Feeds/ }))
    expect(await screen.findByRole("heading", { name: "Feeds" })).toBeInTheDocument()
    await user.click(screen.getByRole("button", { name: /PDF review/ }))
    await waitFor(() => expect(window.location.pathname).toMatch(/^\/manage\/pdf-review/))
    await screen.findByText("Ada Lovelace · 2026 · November")

    await user.click(screen.getByRole("button", { name: /Sign out/ }))
    expect(await screen.findByRole("heading", { name: "Welcome back" })).toBeInTheDocument()
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith("/api/management/session", expect.objectContaining({ method: "DELETE" })))
  })

  it("opens a selected feed and PDF review month directly from their URLs", async () => {
    installApi(true)
    window.history.replaceState({}, "", "/manage/feeds/feed-1")

    const { unmount } = render(<App />)
    expect(await screen.findByRole("region", { name: "Feed details" })).toBeInTheDocument()
    expect(window.location.pathname).toBe("/manage/feeds/feed-1")

    unmount()
    window.history.replaceState({}, "", "/manage/pdf-review/feed-1/2026/month-1")
    render(<App />)

    expect(await screen.findByRole("heading", { name: "PDF review" })).toBeInTheDocument()
    expect(await screen.findByText("Ada Lovelace · 2026 · November")).toBeInTheDocument()
    expect(window.location.pathname).toBe("/manage/pdf-review/feed-1/2026/month-1")
  })

  it("returns from PDF review to the selected feed", async () => {
    installApi(true)
    window.history.replaceState({}, "", "/manage/pdf-review/feed-1/2026/month-1")
    const user = userEvent.setup()
    render(<App />)

    await screen.findByText("Ada Lovelace · 2026 · November")
    await user.click(screen.getByRole("button", { name: "Back to feeds" }))

    expect(window.location.pathname).toBe("/manage/feeds/feed-1")
    expect(await screen.findByRole("region", { name: "Feed details" })).toBeInTheDocument()
  })

  it("updates the browser URL when selecting a feed and returns to Overview from the brand", async () => {
    installApi(true)
    const user = userEvent.setup()
    render(<App />)

    await screen.findByRole("heading", { name: "Overview" })
    const nav = screen.getByRole("navigation", { name: "Primary navigation" })
    await user.click(within(nav).getByRole("button", { name: /Feeds/ }))
    expect(window.location.pathname).toBe("/manage/feeds")
    await user.click(await screen.findByRole("button", { name: /Ada Lovelace/ }))
    expect(window.location.pathname).toBe("/manage/feeds/feed-1")
    await user.click(screen.getAllByRole("link", { name: "Kpapers overview" })[0])
    expect(window.location.pathname).toBe("/manage/")
    window.history.back()
    await waitFor(() => expect(window.location.pathname).toBe("/manage/feeds/feed-1"))
  })

  it("recovers to the review chooser when a bookmarked month is no longer available", async () => {
    installApi(true)
    window.history.replaceState({}, "", "/manage/pdf-review/feed-1/2026/removed-month")
    render(<App />)

    await screen.findByRole("heading", { name: "PDF review" })
    await waitFor(() => expect(window.location.pathname).toBe("/manage/pdf-review"))
    expect(await screen.findByText("The requested review month is no longer available.")).toBeInTheDocument()
  })

  it("keeps PDF review feed, year, and month selections in the URL", async () => {
    installApi(true)
    const user = userEvent.setup()
    render(<App />)

    await screen.findByRole("heading", { name: "Overview" })
    const nav = screen.getByRole("navigation", { name: "Primary navigation" })
    await user.click(within(nav).getByRole("button", { name: /PDF review/ }))
    await screen.findByText("Ada Lovelace · 2026 · November")
    await waitFor(() => expect(window.location.pathname).toBe("/manage/pdf-review/feed-1/2026/month-1"))
    await screen.findByText("No PDFs found")
    await user.selectOptions(screen.getByRole("combobox", { name: "Feed" }), "feed-2")

    await waitFor(() => {
      expect(screen.getByRole("combobox", { name: "Feed" })).toHaveValue("feed-2")
      expect(screen.getByRole("combobox", { name: "Year" })).toHaveValue("2025")
      expect(screen.getByRole("combobox", { name: "Month" })).toHaveValue("month-2")
    })
    await waitFor(() => expect(window.location.pathname).toBe("/manage/pdf-review/feed-2/2025/month-2"))
  })

  it("persists the collapsible desktop sidebar state across reloads", async () => {
    localStorage.removeItem("kpapers.sidebar.collapsed.v1")
    installApi(true)
    const user = userEvent.setup()
    const firstRender = render(<App />)
    await screen.findByRole("heading", { name: "Overview" })

    const collapseButton = screen.getByRole("button", { name: "Collapse sidebar" })
    expect(collapseButton).toHaveAttribute("aria-expanded", "true")
    collapseButton.focus()
    await user.keyboard("{Enter}")
    expect(screen.getByRole("button", { name: "Expand sidebar" })).toHaveAttribute("aria-expanded", "false")
    expect(localStorage.getItem("kpapers.sidebar.collapsed.v1")).toBe("collapsed")

    firstRender.unmount()
    render(<App />)
    await screen.findByRole("heading", { name: "Overview" })
    expect(screen.getByRole("button", { name: "Expand sidebar" })).toHaveAttribute("aria-expanded", "false")
  })

  it("shows zero counts when there are no source folders or PDFs needing review", async () => {
    installApi(true, async () => response({ source_folder_count: 0, needs_review_count: 0 }))
    render(<App />)

    await screen.findByRole("heading", { name: "Overview" })
    await waitFor(() => {
      expect(screen.getByTestId("source-folder-count")).toHaveTextContent("0")
      expect(screen.getByTestId("needs-review-count")).toHaveTextContent("0")
      expect(screen.getByText("No documents need attention")).toBeInTheDocument()
    })
  })

  it("shows loading and an unavailable state for Overview count requests", async () => {
    let resolveSummary!: (value: Response) => void
    const pendingSummary = () => new Promise<Response>((resolve) => { resolveSummary = resolve })
    installApi(true, pendingSummary)
    const { unmount } = render(<App />)

    await screen.findByRole("heading", { name: "Overview" })
    expect(screen.getByTestId("source-folder-count")).toHaveTextContent("Loading…")
    expect(screen.getByTestId("needs-review-count")).toHaveTextContent("Loading…")
    resolveSummary(response({ source_folder_count: 4, needs_review_count: 1 }))
    await waitFor(() => expect(screen.getByTestId("source-folder-count")).toHaveTextContent("4"))

    unmount()
    installApi(true, async () => response(null, 503))
    render(<App />)
    await screen.findByRole("heading", { name: "Overview" })
    await waitFor(() => {
      expect(screen.getByTestId("source-folder-count")).toHaveTextContent("Unavailable")
      expect(screen.getByTestId("needs-review-count")).toHaveTextContent("Unavailable")
      expect(screen.getByText(/Review queue unavailable/)).toBeInTheDocument()
      expect(screen.queryByText("No documents need attention")).not.toBeInTheDocument()
    })
  })

  it.each(["/", "/manage", "/manage/"])("normalizes the Overview entry %s", async (path) => {
    installApi(true)
    window.history.replaceState({}, "", path)
    render(<App />)
    await screen.findByRole("heading", { name: "Overview" })
    expect(window.location.pathname).toBe("/manage/")
  })

  it.each(["/manage/pdf-review/feed-1", "/manage/pdf-review/feed-1/2026"])("restores partial review selection %s", async (path) => {
    installApi(true)
    window.history.replaceState({}, "", path)
    render(<App />)
    // Partial URLs are completed in stages, remounting review at each route.
    // Query the current DOM inside waitFor instead of retaining a removed badge.
    await waitFor(() => {
      expect(window.location.pathname).toBe("/manage/pdf-review/feed-1/2026/month-1")
      expect(screen.getByRole("combobox", { name: "Feed" })).toHaveValue("feed-1")
      expect(screen.getByRole("combobox", { name: "Year" })).toHaveValue("2026")
      expect(screen.getByRole("combobox", { name: "Month" })).toHaveValue("month-1")
      expect(screen.getByText("Ada Lovelace · 2026 · November")).toBeInTheDocument()
      expect(screen.getByText("No PDFs found")).toBeInTheDocument()
    })
  })

  it("recovers from a missing feed and an unknown dashboard path", async () => {
    installApi(true)
    window.history.replaceState({}, "", "/manage/feeds/missing")
    const first = render(<App />)
    await screen.findByText("That feed is no longer available.")
    expect(window.location.pathname).toBe("/manage/feeds")
    first.unmount()
    window.history.replaceState({}, "", "/manage/unknown")
    render(<App />)
    await screen.findByRole("heading", { name: "Overview" })
    expect(window.location.pathname).toBe("/manage/")
  })

  it("opens protected creation after sign-in without treating new as a feed ID", async () => {
    installApi(false)
    window.history.replaceState({}, "", "/manage/feeds/new")
    const user = userEvent.setup()
    render(<App />)
    await screen.findByLabelText("Username")
    expect(screen.queryByLabelText("Author name")).not.toBeInTheDocument()
    await user.type(screen.getByLabelText("Username"), "operator")
    await user.type(screen.getByLabelText("Password"), "correct horse")
    await user.click(screen.getByRole("button", { name: "Sign in" }))
    expect(await screen.findByRole("heading", { name: "Create a feed" })).toBeInTheDocument()
    expect(window.location.pathname).toBe("/manage/feeds/new")
    expect(screen.getByRole("button", { name: /Feeds —/ })).toHaveAttribute("aria-current", "page")
    expect(screen.queryByText("That feed is no longer available.")).not.toBeInTheDocument()
  })

  it("opens creation from Overview, cancels without a POST, and restores it with history", async () => {
    const fetchMock = installApi(true)
    const user = userEvent.setup()
    render(<App />)
    await user.click(await screen.findByRole("button", { name: "Create feed" }))
    await screen.findByLabelText("Author name")
    expect(window.location.pathname).toBe("/manage/feeds/new")
    await user.click(screen.getByRole("button", { name: "Cancel" }))
    await screen.findByRole("heading", { name: "Feeds" })
    expect(screen.queryByLabelText("Author name")).not.toBeInTheDocument()
    expect(fetchMock.mock.calls.some(([, init]) => init?.method === "POST")).toBe(false)
    window.history.back()
    await screen.findByRole("heading", { name: "Create a feed" })
    window.history.forward()
    await screen.findByRole("heading", { name: "Feeds" })
  })

})

describe("Overview onboarding", () => {
  it("shows first-feed guidance only when there are no active feeds", async () => {
    const fetchMock = installApi(true)
    const original = fetchMock.getMockImplementation()!
    fetchMock.mockImplementation(async (input, init) => String(input) === "/api/management/feeds" ? response([]) : original(input, init))
    render(<App />)
    await screen.findByRole("heading", { name: "Overview" })
    expect(screen.getByText("Get started")).toBeInTheDocument()
    expect(screen.getByRole("button", { name: "Create your first feed" })).toBeInTheDocument()
    expect(screen.queryByText("Publishing workflow")).not.toBeInTheDocument()
  })
})
