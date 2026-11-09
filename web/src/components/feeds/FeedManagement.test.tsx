import { render, screen, waitFor, within } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { beforeEach, describe, expect, it, vi } from "vitest"
import App from "@/App"

type TestFeed = {
  id: string
  display_name: string
  description: string
  active: boolean
  refresh_interval_seconds: number | null
}

const feed: TestFeed = {
  id: "feed-1", display_name: "Ada Lovelace", description: "Research notes", active: true, refresh_interval_seconds: null,
}

function json(body: unknown, status = 200) {
  return new Response(body === null ? null : JSON.stringify(body), {
    status,
    headers: body === null ? {} : { "Content-Type": "application/json" },
  })
}

function installFeedApi(initial: TestFeed[] = []) {
  window.history.replaceState({}, "", "/")
  let feeds = structuredClone(initial)
  let years: { feed_id: string; year: number; active: boolean }[] = []
  let months: { id: string; feed_id: string; year: number; month: number; provider: string; source_url: string; active: boolean; last_attempt_at: string | null; next_refresh_at: string | null; last_result: string | null; last_error: string | null }[] = []
  const fetchMock = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
    const path = String(input)
    const method = init?.method ?? "GET"
    if (path === "/api/management/session") return json({ status: "authenticated" })
    if (path === "/api/management/feeds" && method === "GET") return json(feeds)
    if (path === "/api/management/feeds" && method === "POST") {
      const payload = JSON.parse(String(init?.body)) as Omit<TestFeed, "id" | "active">
      if (!payload.display_name) return json(null, 400)
      const created: TestFeed = { ...payload, id: "feed-new", active: true }
      feeds = [...feeds, created]
      return json(created, 201)
    }
    if (path === "/api/management/feeds/feed-1/years" && method === "GET") return json(years)
    if (path === "/api/management/feeds/feed-1/years" && method === "POST") {
      const payload = JSON.parse(String(init?.body)) as { year: number }
      const created = { feed_id: "feed-1", year: payload.year, active: true }
      years = [...years, created]
      return json(created, 201)
    }
    const yearMonths = path.match(/^\/api\/management\/feeds\/feed-1\/years\/(\d+)\/months$/)
    if (yearMonths && method === "GET") return json(months.filter((month) => month.year === Number(yearMonths[1])))
    const linkMonth = path.match(/^\/api\/management\/feeds\/feed-1\/years\/(\d+)\/months\/(\d+)$/)
    if (linkMonth && method === "PUT") {
      const payload = JSON.parse(String(init?.body)) as { source_url: string }
      if (payload.source_url.includes("invalid")) return json(null, 400)
      const created = { id: "month-jan", feed_id: "feed-1", year: Number(linkMonth[1]), month: Number(linkMonth[2]), provider: "yandex_disk", source_url: payload.source_url, active: true, last_attempt_at: "2026-11-07T10:30:00Z", next_refresh_at: "2026-11-08T10:30:00Z", last_result: "success", last_error: null }
      months = [...months, created]
      return json(created, 201)
    }
    const monthAction = path.match(/^\/api\/management\/months\/([^/]+)(?:\/(refresh|discover|archive|restore))?$/)
    if (monthAction) {
      const [, id, action] = monthAction
      const target = months.find((month) => month.id === id)
      if (!target) return json(null, 404)
      if (action === "refresh" && method === "POST") return json({ job_id: "month-job", status: "queued" }, 202)
      if (action === "discover" && method === "POST") return json({ discovered_pdfs: 2 })
      if (action === "archive" && method === "POST") target.active = false
      else if (action === "restore" && method === "POST") target.active = true
      else if (!action && method === "DELETE") months = months.filter((month) => month.id !== id)
      else return json([], 200)
      return json(null, 204)
    }
    if (path === "/api/management/months/month-jan/documents" && method === "GET") return json([])
    const yearAction = path.match(/^\/api\/management\/feeds\/feed-1\/years\/(\d+)(?:\/(archive|restore|refresh))?$/)
    if (yearAction) {
      const [, rawYear, action] = yearAction
      const target = years.find((year) => year.year === Number(rawYear))
      if (!target) return json(null, 404)
      if (action === "refresh" && method === "POST") return json({ job_id: "year-job", status: "queued" }, 202)
      if (action === "archive" && method === "POST") target.active = false
      else if (action === "restore" && method === "POST") target.active = true
      else if (!action && method === "DELETE") years = years.filter((year) => year.year !== Number(rawYear))
      else return json([], 200)
      return json(null, 204)
    }
    const match = path.match(/^\/api\/management\/feeds\/([^/]+)(?:\/(archive|restore|permanent|refresh))?$/)
    if (match) {
      const [, id, action] = match
      const target = feeds.find((item) => item.id === id)
      if (!target) return json(null, 404)
      if (action === "archive" && method === "POST") target.active = false
      else if (action === "restore" && method === "POST") target.active = true
      else if (action === "permanent" && method === "DELETE") feeds = feeds.filter((item) => item.id !== id)
      else if (action === "refresh" && method === "POST") return json({ job_id: "job-1", status: "queued" }, 202)
      else if (!action && method === "PUT") {
        const payload = JSON.parse(String(init?.body)) as Pick<TestFeed, "display_name" | "description" | "refresh_interval_seconds">
        target.display_name = payload.display_name
        target.description = payload.description
        target.refresh_interval_seconds = payload.refresh_interval_seconds
        return json(target)
      } else return json(null, 404)
      return json(null, 204)
    }
    return json(null, 404)
  })
  vi.stubGlobal("fetch", fetchMock)
  return fetchMock
}

async function openFeeds(user: ReturnType<typeof userEvent.setup>) {
  await screen.findByRole("heading", { name: "Overview" })
  const nav = screen.getByRole("navigation", { name: "Primary navigation" })
  await user.click(within(nav).getByRole("button", { name: /Feeds/ }))
  await screen.findByRole("heading", { name: "Feeds" })
  const feedButton = screen.queryByRole("button", { name: /Ada Lovelace/ })
  if (feedButton) {
    await user.click(feedButton)
    await screen.findByRole("region", { name: "Feed details" })
  }
}

describe("feed management workflows", () => {
  beforeEach(() => localStorage.clear())
  it("copies the RSS address with the Clipboard API when available", async () => {
    installFeedApi([feed])
    const writeText = vi.fn().mockResolvedValue(undefined)
    const user = userEvent.setup()
    const previousClipboard = Object.getOwnPropertyDescriptor(window.navigator, "clipboard")
    Object.defineProperty(window.navigator, "clipboard", { configurable: true, value: { writeText } })
    try {
      render(<App />)
      await openFeeds(user)
      await user.click(screen.getByRole("button", { name: "Copy RSS address" }))

      expect(await screen.findByText("RSS address copied")).toBeInTheDocument()
      expect(writeText).toHaveBeenCalledWith(`${window.location.origin}/feeds/feed-1`)
      expect(await screen.findByRole("button", { name: "Copy RSS address" })).toHaveTextContent("Copied")
    } finally {
      if (previousClipboard) Object.defineProperty(window.navigator, "clipboard", previousClipboard)
      else delete (window.navigator as Navigator & { clipboard?: Clipboard }).clipboard
    }
  })

  it("uses the document copy fallback when the Clipboard API is unavailable", async () => {
    installFeedApi([feed])
    const user = userEvent.setup()
    const previousClipboard = Object.getOwnPropertyDescriptor(window.navigator, "clipboard")
    Object.defineProperty(window.navigator, "clipboard", { configurable: true, value: undefined })
    const previousExecCommand = Object.getOwnPropertyDescriptor(document, "execCommand")
    const execCommand = vi.fn().mockReturnValue(true)
    Object.defineProperty(document, "execCommand", { configurable: true, value: execCommand })
    try {
      render(<App />)
      await openFeeds(user)
      await user.click(screen.getByRole("button", { name: "Copy RSS address" }))

      expect(execCommand).toHaveBeenCalledWith("copy")
      expect(await screen.findByText("RSS address copied")).toBeInTheDocument()
      expect(document.querySelector("textarea[readonly]")).not.toBeInTheDocument()
    } finally {
      if (previousClipboard) Object.defineProperty(window.navigator, "clipboard", previousClipboard)
      else delete (window.navigator as Navigator & { clipboard?: Clipboard }).clipboard
      if (previousExecCommand) Object.defineProperty(document, "execCommand", previousExecCommand)
      else delete (document as Document & { execCommand?: (command: string) => boolean }).execCommand
    }
  })

  it("falls back after a Clipboard API rejection and reports failure when both methods fail", async () => {
    installFeedApi([feed])
    const user = userEvent.setup()
    const previousClipboard = Object.getOwnPropertyDescriptor(window.navigator, "clipboard")
    Object.defineProperty(window.navigator, "clipboard", { configurable: true, value: { writeText: vi.fn().mockRejectedValue(new Error("denied")) } })
    const previousExecCommand = Object.getOwnPropertyDescriptor(document, "execCommand")
    const execCommand = vi.fn().mockReturnValue(false)
    Object.defineProperty(document, "execCommand", { configurable: true, value: execCommand })
    try {
      render(<App />)
      await openFeeds(user)
      await user.click(screen.getByRole("button", { name: "Copy RSS address" }))

      expect(execCommand).toHaveBeenCalledWith("copy")
      expect(await screen.findByText("Could not copy the RSS address. Select and copy it instead.")).toBeInTheDocument()
      expect(screen.getByRole("button", { name: "Copy RSS address" })).toHaveTextContent("Copy link")
    } finally {
      if (previousClipboard) Object.defineProperty(window.navigator, "clipboard", previousClipboard)
      else delete (window.navigator as Navigator & { clipboard?: Clipboard }).clipboard
      if (previousExecCommand) Object.defineProperty(document, "execCommand", previousExecCommand)
      else delete (document as Document & { execCommand?: (command: string) => boolean }).execCommand
    }
  })

  it("creates a feed with a trimmed name and an hours-to-seconds refresh interval", async () => {
    const fetchMock = installFeedApi()
    const user = userEvent.setup()
    render(<App />)
    await openFeeds(user)
    await user.click(screen.getByRole("button", { name: "Create feed" }))

    await user.type(screen.getByLabelText("Author name"), "  Grace Hopper  ")
    await user.type(screen.getByLabelText(/Description/), "  Compiler notes  ")
    await user.type(screen.getByLabelText(/Refresh interval/), "1.5")
    await user.click(screen.getByRole("button", { name: "Create feed" }))

    await screen.findByRole("heading", { name: "Grace Hopper" })
    const createCall = fetchMock.mock.calls.find(([path, init]) => String(path) === "/api/management/feeds" && init?.method === "POST")
    expect(JSON.parse(String(createCall?.[1]?.body))).toEqual({
      display_name: "Grace Hopper", description: "Compiler notes", refresh_interval_seconds: 5400,
    })
    expect(screen.getByText("Stable RSS address")).toBeInTheDocument()
    expect(screen.getByText(/\/feeds\/feed-new$/)).toBeInTheDocument()
  })

  it("shows validation feedback when feed creation fails", async () => {
    installFeedApi()
    const user = userEvent.setup()
    render(<App />)
    await openFeeds(user)
    await user.click(screen.getByRole("button", { name: "Create feed" }))

    await user.type(screen.getByLabelText("Author name"), "Octavia Butler")
    await user.type(screen.getByLabelText(/Refresh interval/), "0")
    await user.click(screen.getByRole("button", { name: "Create feed" }))

    expect(await screen.findByTestId("feed-error-summary")).toHaveTextContent("Refresh interval must be greater than zero hours.")
  })

  it("blocks refresh intervals above the supported form maximum", async () => {
    const fetchMock = installFeedApi()
    const user = userEvent.setup()
    render(<App />)
    await openFeeds(user)
    await user.click(screen.getByRole("button", { name: "Create feed" }))

    const interval = screen.getByLabelText(/Refresh interval/) as HTMLInputElement
    await user.type(screen.getByLabelText("Author name"), "Katherine Johnson")
    await user.type(interval, "87601")
    expect(interval.validity.rangeOverflow).toBe(true)
    await user.click(screen.getByRole("button", { name: "Create feed" }))

    expect(fetchMock).not.toHaveBeenCalledWith("/api/management/feeds", expect.objectContaining({ method: "POST" }))
    expect(screen.queryByRole("heading", { name: "Katherine Johnson" })).not.toBeInTheDocument()
  })

  it("edits feed details, keeps the stable address, and requires confirmation before archiving and deletion", async () => {
    const fetchMock = installFeedApi([feed])
    const user = userEvent.setup()
    render(<App />)
    await openFeeds(user)

    await user.click(screen.getByRole("button", { name: "Edit details" }))
    const details = screen.getByRole("dialog", { name: "Edit feed details" })
    const nameInput = within(details).getByLabelText("Author name")
    await user.clear(nameInput)
    await user.type(nameInput, "Ada Lovelace Journal")
    await user.click(within(details).getByRole("button", { name: "Save changes" }))

    expect(await screen.findByRole("heading", { name: "Ada Lovelace Journal" })).toBeInTheDocument()
    expect(screen.getByText(/\/feeds\/feed-1$/)).toBeInTheDocument()
    expect(fetchMock).toHaveBeenCalledWith("/api/management/feeds/feed-1", expect.objectContaining({ method: "PUT" }))

    await user.click(screen.getByRole("button", { name: "More actions for Ada Lovelace Journal" }))
    await user.click(screen.getByRole("menuitem", { name: "Archive feed" }))
    expect(screen.getByRole("alertdialog")).toHaveTextContent("will be hidden from refresh and RSS")
    await user.click(screen.getByRole("button", { name: "Archive feed" }))
    expect(await screen.findByText("Archived")).toBeInTheDocument()

    await user.click(screen.getByRole("button", { name: "Delete permanently" }))
    expect(screen.getByRole("alertdialog")).toHaveTextContent("This cannot be undone.")
    await user.click(screen.getByRole("button", { name: "Cancel" }))
    expect(screen.getByRole("heading", { name: "Ada Lovelace Journal" })).toBeInTheDocument()

    await user.click(screen.getByRole("button", { name: "Delete permanently" }))
    const dialog = screen.getByRole("alertdialog")
    await user.click(within(dialog).getByRole("button", { name: "Delete permanently" }))
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith("/api/management/feeds/feed-1/permanent", expect.objectContaining({ method: "DELETE" })))
  })

  it("links a month, displays refresh metadata, and supports scoped refresh and review", async () => {
    const fetchMock = installFeedApi([feed])
    const user = userEvent.setup()
    render(<App />)
    await openFeeds(user)
    await screen.findByRole("region", { name: "Feed details" })

    await user.click(screen.getByRole("button", { name: "Add year" }))
    await user.type(screen.getByLabelText("Add a year"), "2026")
    await user.click(within(screen.getByRole("dialog")).getByRole("button", { name: "Add year" }))
    await screen.findByText("2026", { selector: "*[data-slot='card-title']" })
    await user.click(screen.getByRole("button", { name: "Expand 2026" }))
    await user.click(screen.getByRole("button", { name: "Connect month" }))
    await user.selectOptions(screen.getByLabelText("Month"), "1")
    await user.type(screen.getByLabelText("Public folder link"), "https://disk.yandex.ru/example")
    await user.click(screen.getByRole("button", { name: "Link month" }))

    expect(await screen.findByText("January", { selector: "h4" })).toBeInTheDocument()
    expect(fetchMock).toHaveBeenCalledWith("/api/management/feeds/feed-1/years/2026/months/1", expect.objectContaining({
      method: "PUT", body: JSON.stringify({ source_url: "https://disk.yandex.ru/example" }),
    }))
    await user.click(screen.getByRole("button", { name: "Manage source" }))
    await user.click(screen.getByRole("button", { name: "Refresh now" }))
    expect(await screen.findByText("January refresh queued")).toBeInTheDocument()
    expect(fetchMock).toHaveBeenCalledWith("/api/management/months/month-jan/refresh", expect.objectContaining({ method: "POST" }))
    await user.click(screen.getByRole("button", { name: "Review PDFs" }))
    expect(await screen.findByRole("heading", { name: "PDF review" })).toBeInTheDocument()
  })

  it("shows an API error when a month source link cannot be saved", async () => {
    installFeedApi([feed])
    const user = userEvent.setup()
    render(<App />)
    await openFeeds(user)
    await user.click(screen.getByRole("button", { name: "Add year" }))
    await user.type(screen.getByLabelText("Add a year"), "2025")
    await user.click(within(screen.getByRole("dialog")).getByRole("button", { name: "Add year" }))
    await screen.findByText("2025", { selector: "*[data-slot='card-title']" })
    await user.click(screen.getByRole("button", { name: "Expand 2025" }))
    await user.click(screen.getByRole("button", { name: "Connect month" }))
    await user.selectOptions(screen.getByLabelText("Month"), "2")
    await user.type(screen.getByLabelText("Public folder link"), "https://invalid.example")
    await user.click(screen.getByRole("button", { name: "Link month" }))
    expect(await screen.findByRole("alert")).toHaveTextContent("Please check the information and try again.")
  })

  it("keeps entered values after an API error and retries creation without a follow-up list request", async () => {
    const fetchMock = installFeedApi()
    const original = fetchMock.getMockImplementation()!
    let posts = 0
    fetchMock.mockImplementation(async (input, init) => {
      if (String(input) === "/api/management/feeds" && init?.method === "POST") {
        posts += 1
        if (posts === 1) return json(null, 500)
      }
      // A subsequent list read is unavailable; successful creation must still open its details.
      if (posts > 1 && String(input) === "/api/management/feeds" && !init?.method) return json(null, 503)
      return original(input, init)
    })
    const user = userEvent.setup()
    render(<App />)
    await user.click(await screen.findByRole("button", { name: "Create feed" }))
    await user.type(screen.getByLabelText("Author name"), "Retry author")
    await user.type(screen.getByLabelText(/Description/), "Keep these notes")
    await user.click(screen.getByRole("button", { name: "Create feed" }))
    expect(await screen.findByRole("alert")).toBeInTheDocument()
    expect(screen.getByLabelText("Author name")).toHaveValue("Retry author")
    expect(screen.getByLabelText(/Description/)).toHaveValue("Keep these notes")
    expect(window.location.pathname).toBe("/manage/feeds/new")
    await user.click(screen.getByRole("button", { name: "Create feed" }))
    await screen.findByRole("heading", { name: "Retry author" })
    expect(window.location.pathname).toBe("/manage/feeds/feed-new")
    expect(posts).toBe(2)
    expect(screen.queryByText("That feed is no longer available.")).not.toBeInTheDocument()
    expect(fetchMock.mock.calls.filter(([path, init]) => String(path) === "/api/management/feeds" && !init?.method)).toHaveLength(1)
  })

})

describe("approved feed screens", () => {
  beforeEach(() => localStorage.clear())

  it("keeps hierarchy off the list and offers archived feed restoration through its menu", async () => {
    const archived = { ...feed, id: "archived", display_name: "Archived author", active: false }
    const fetchMock = installFeedApi([feed, archived])
    const user = userEvent.setup()
    render(<App />)
    await screen.findByRole("heading", { name: "Overview" })
    await user.click(screen.getByRole("button", { name: /Feeds —/ }))
    expect(screen.queryByRole("region", { name: "Feed details" })).not.toBeInTheDocument()
    expect(screen.queryByRole("button", { name: "Add year" })).not.toBeInTheDocument()
    await user.click(screen.getByRole("tab", { name: "Archived (1)" }))
    await user.click(screen.getByRole("button", { name: "Actions for Archived author" }))
    await user.click(screen.getByRole("menuitem", { name: "Restore" }))
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith("/api/management/feeds/archived/restore", expect.objectContaining({ method: "POST" })))
    await user.click(screen.getByRole("tab", { name: "Active (2)" }))
    expect(screen.getByRole("button", { name: /Archived author/ })).toBeInTheDocument()
  })

  it("focuses linked validation errors and returns focus when editing is cancelled with Escape", async () => {
    installFeedApi([feed])
    const user = userEvent.setup()
    render(<App />)
    await openFeeds(user)
    const edit = screen.getByRole("button", { name: "Edit details" })
    await user.click(edit)
    await user.clear(screen.getByLabelText("Author name"))
    await user.click(screen.getByRole("button", { name: "Save changes" }))
    const summary = await screen.findByTestId("feed-error-summary")
    await waitFor(() => expect(summary).toHaveFocus())
    const field = screen.getByLabelText("Author name")
    expect(field).toHaveAttribute("aria-invalid", "true")
    await user.click(within(summary).getByRole("link", { name: "Enter an author name." }))
    expect(field).toHaveFocus()
    expect(field).toHaveAccessibleDescription("Enter an author name.")
    await user.keyboard("{Escape}")
    await waitFor(() => expect(edit).toHaveFocus())
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument()
  })

  it("disables submission while creating and opens the returned feed after it completes", async () => {
    const fetchMock = installFeedApi()
    const original = fetchMock.getMockImplementation()!
    let finish!: () => void
    fetchMock.mockImplementation(async (input, init) => {
      if (String(input) === "/api/management/feeds" && init?.method === "POST") await new Promise<void>((resolve) => { finish = resolve })
      return original(input, init)
    })
    const user = userEvent.setup()
    render(<App />)
    await user.click(await screen.findByRole("button", { name: "Create feed" }))
    await user.type(screen.getByLabelText("Author name"), "Pending author")
    await user.click(screen.getByRole("button", { name: "Create feed" }))
    expect(screen.getByRole("button", { name: "Saving…" })).toBeDisabled()
    finish()
    expect(await screen.findByRole("heading", { name: "Pending author" })).toBeInTheDocument()
  })
})
