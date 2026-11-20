import { render, screen, waitFor, within } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"
import { api, type Month } from "@/lib/api"
import { HierarchyManagement } from "./HierarchyManagement"

const month: Month = {
  id: "month-1", feed_id: "feed-1", year: 2026, month: 11, provider: "fixture", source_url: "https://disk.yandex.ru/example",
  active: true, last_attempt_at: null, next_refresh_at: null, last_result: null, last_error: null,
}
const props = { feedId: "feed-1", feedName: "Ada Lovelace", active: true, onReviewMonth: vi.fn() }
const key = (feedId: string, year: number) => `kpapers.year.expanded.v1.${feedId}.${year}`

beforeEach(() => {
  localStorage.clear()
  vi.spyOn(api, "years").mockImplementation(async (feedId) => [
    { feed_id: feedId, year: 2026, active: true },
    { feed_id: feedId, year: 2025, active: false },
  ])
  vi.spyOn(api, "months").mockResolvedValue([month])
  vi.spyOn(api, "exportMonth").mockResolvedValue()
  vi.spyOn(api, "exportYear").mockResolvedValue()
})
afterEach(() => vi.restoreAllMocks())

describe("feed year expansion", () => {
  it("starts active and archived years collapsed, supports independent keyboard toggles, and keeps year actions visible", async () => {
    const user = userEvent.setup()
    render(<HierarchyManagement {...props} />)
    const first = await screen.findByRole("button", { name: "Expand 2026" })
    expect(first).toHaveAttribute("aria-expanded", "false")
    expect(screen.getByRole("button", { name: "Expand 2025" })).toHaveAttribute("aria-expanded", "false")
    expect(screen.queryByRole("textbox", { name: "Public folder link" })).not.toBeInTheDocument()
    await user.click(screen.getByRole("button", { name: "Actions for 2026" }))
    expect(screen.getByRole("menuitem", { name: "Refresh year" })).toBeVisible()
    await user.keyboard("{Escape}")
    await user.click(screen.getByRole("button", { name: "Actions for 2025" }))
    expect(screen.getByRole("menuitem", { name: "Restore year" })).toBeVisible()
    await user.keyboard("{Escape}")
    first.focus()
    await user.keyboard("{Enter}")
    expect(screen.getByRole("button", { name: "Collapse 2026" })).toHaveAttribute("aria-expanded", "true")
    expect(document.getElementById(first.getAttribute("aria-controls")!)).toBeVisible()
    await user.click(screen.getByRole("button", { name: "Expand 2025" }))
    expect(screen.getByRole("button", { name: "Connect month" })).toBeVisible()
    expect(screen.queryByRole("textbox", { name: "Public folder link" })).not.toBeInTheDocument()
    await user.click(screen.getByRole("button", { name: "Collapse 2026" }))
    expect(screen.getByRole("button", { name: "Collapse 2025" })).toHaveAttribute("aria-expanded", "true")
    expect(screen.queryByRole("button", { name: "Connect month" })).not.toBeInTheDocument()
  })

  it("remembers mixed states after remount and isolates the same year when switching feeds", async () => {
    const user = userEvent.setup()
    const view = render(<HierarchyManagement {...props} />)
    await user.click(await screen.findByRole("button", { name: "Expand 2026" }))
    await user.click(screen.getByRole("button", { name: "Expand 2025" }))
    await user.click(screen.getByRole("button", { name: "Collapse 2025" }))
    expect(localStorage.getItem(key("feed-1", 2026))).toBe("expanded")
    expect(localStorage.getItem(key("feed-1", 2025))).toBe("collapsed")
    view.unmount()
    const restored = render(<HierarchyManagement {...props} />)
    expect(await screen.findByRole("button", { name: "Collapse 2026" })).toHaveAttribute("aria-expanded", "true")
    expect(screen.getByRole("button", { name: "Expand 2025" })).toHaveAttribute("aria-expanded", "false")
    restored.rerender(<HierarchyManagement {...props} feedId="feed-2" />)
    await waitFor(() => expect(screen.getByRole("button", { name: "Expand 2026" })).toHaveAttribute("aria-expanded", "false"))
    await user.click(screen.getByRole("button", { name: "Expand 2026" }))
    await user.click(screen.getByRole("button", { name: "Collapse 2026" }))
    restored.rerender(<HierarchyManagement {...props} />)
    expect(await screen.findByRole("button", { name: "Collapse 2026" })).toHaveAttribute("aria-expanded", "true")
  })

  it("defaults malformed preferences to collapsed and remains usable with blocked reads and writes", async () => {
    const user = userEvent.setup()
    localStorage.setItem(key("feed-1", 2026), "invalid")
    const view = render(<HierarchyManagement {...props} />)
    expect(await screen.findByRole("button", { name: "Expand 2026" })).toHaveAttribute("aria-expanded", "false")
    view.unmount()
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => { throw new Error("blocked") })
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => { throw new Error("blocked") })
    render(<HierarchyManagement {...props} />)
    await user.click(await screen.findByRole("button", { name: "Expand 2026" }))
    expect(screen.getByRole("button", { name: "Collapse 2026" })).toHaveAttribute("aria-expanded", "true")
    await user.click(screen.getByRole("button", { name: "Collapse 2026" }))
    expect(screen.getByRole("button", { name: "Expand 2026" })).toHaveAttribute("aria-expanded", "false")
  })
})

describe("month source details", () => {
  it("exports a month and its year from their hierarchy actions", async () => {
    const user = userEvent.setup()
    render(<HierarchyManagement {...props} />)
    await user.click(await screen.findByRole("button", { name: "Expand 2026" }))

    const monthRow = within(document.getElementById("year-content-feed-1-2026")!).getByRole("article")
    await user.click(within(monthRow).getByRole("button", { name: "Export Markdown" }))
    expect(api.exportMonth).toHaveBeenCalledWith("month-1", "Ada Lovelace", 2026, 11)

    await user.click(screen.getByRole("button", { name: "Actions for 2026" }))
    await user.click(screen.getByRole("menuitem", { name: "Export Markdown" }))
    expect(api.exportYear).toHaveBeenCalledWith("feed-1", 2026, "Ada Lovelace")
  })

  it("hides scoped export actions for an archived feed", async () => {
    const user = userEvent.setup()
    render(<HierarchyManagement {...props} active={false} />)
    await user.click(await screen.findByRole("button", { name: "Expand 2026" }))
    const monthRow = within(document.getElementById("year-content-feed-1-2026")!).getByRole("article")
    expect(within(monthRow).queryByRole("button", { name: "Export Markdown" })).not.toBeInTheDocument()
    await user.click(screen.getByRole("button", { name: "Actions for 2026" }))
    expect(screen.queryByRole("menuitem", { name: "Export Markdown" })).not.toBeInTheDocument()
  })

  it("keeps the month summary scannable and discloses its source, refresh state, and scoped actions", async () => {
    const user = userEvent.setup()
    render(<HierarchyManagement {...props} />)
    await user.click(await screen.findByRole("button", { name: "Expand 2026" }))

    const yearContent = within(document.getElementById("year-content-feed-1-2026")!)
    const monthRow = yearContent.getByRole("article")
    expect(within(monthRow).getByRole("heading", { name: "November" })).toBeVisible()
    expect(within(monthRow).getByText("Not refreshed")).toBeVisible()
    expect(within(monthRow).getByRole("button", { name: "Review PDFs" })).toBeVisible()
    const disclosure = within(monthRow).getByRole("button", { name: "Show source details for November" })
    expect(disclosure).toHaveAttribute("aria-expanded", "false")
    expect(document.getElementById(disclosure.getAttribute("aria-controls")!)).not.toBeVisible()

    await user.click(disclosure)
    expect(within(monthRow).getByRole("button", { name: "Hide source details for November" })).toHaveAttribute("aria-expanded", "true")
    expect(within(monthRow).getByRole("link", { name: month.source_url })).toBeVisible()
    expect(within(monthRow).getByText("Never")).toBeVisible()
    expect(within(monthRow).getByText("Pending")).toBeVisible()
    expect(within(monthRow).getByText("Not run yet")).toBeVisible()
    expect(within(monthRow).getByRole("button", { name: "Fetch and parse" })).toBeVisible()
    expect(within(monthRow).getByRole("button", { name: "Refresh now" })).toBeVisible()
    expect(within(monthRow).getByRole("button", { name: "Archive" })).toBeVisible()
    expect(within(monthRow).getByRole("button", { name: "Delete permanently" })).toBeVisible()
  })

  it("shows a failed refresh and its error in the disclosed details", async () => {
    vi.mocked(api.months).mockResolvedValue([{
      ...month,
      last_attempt_at: "2026-11-18T09:35:24Z",
      next_refresh_at: "2026-11-18T10:35:22Z",
      last_result: "failure",
      last_error: "Folder unavailable",
    }])
    const user = userEvent.setup()
    render(<HierarchyManagement {...props} />)
    await user.click(await screen.findByRole("button", { name: "Expand 2026" }))
    const monthRow = within(document.getElementById("year-content-feed-1-2026")!).getByRole("article")
    expect(within(monthRow).getByText("failure")).toBeVisible()
    await user.click(within(monthRow).getByRole("button", { name: "Show source details for November" }))
    expect(within(monthRow).getByText("Failed · Folder unavailable")).toBeVisible()
  })
})


describe("source folder dialogs", () => {
  it("closes focused forms with Escape and returns focus after cancelling year deletion", async () => {
    const user = userEvent.setup()
    render(<HierarchyManagement {...props} />)
    const add = screen.getByRole("button", { name: "Add year" })
    await user.click(add)
    expect(screen.getByRole("dialog", { name: "Add year" })).toBeInTheDocument()
    await user.keyboard("{Escape}")
    await waitFor(() => expect(add).toHaveFocus())
    await user.click(await screen.findByRole("button", { name: "Expand 2026" }))
    const connect = screen.getByRole("button", { name: "Connect month" })
    await user.click(connect)
    expect(screen.getByRole("dialog", { name: "Connect month in 2026" })).toBeInTheDocument()
    await user.keyboard("{Escape}")
    await waitFor(() => expect(connect).toHaveFocus())
    const actions = screen.getByRole("button", { name: "Actions for 2026" })
    await user.click(actions)
    await user.click(screen.getByRole("menuitem", { name: "Delete year permanently" }))
    expect(screen.getByRole("alertdialog")).toHaveTextContent("This cannot be undone.")
    await user.click(screen.getByRole("button", { name: "Cancel" }))
    await waitFor(() => expect(actions).toHaveFocus())
  })
})
