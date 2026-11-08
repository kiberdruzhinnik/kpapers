import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"
import { api, type Month } from "@/lib/api"
import { HierarchyManagement } from "./HierarchyManagement"

const month: Month = {
  id: "month-1", feed_id: "feed-1", year: 2026, month: 11, provider: "fixture", source_url: "https://disk.yandex.ru/example",
  active: true, last_attempt_at: null, next_refresh_at: null, last_result: null, last_error: null,
}
const props = { feedId: "feed-1", active: true, onReviewMonth: vi.fn() }
const key = (feedId: string, year: number) => `kpapers.year.expanded.v1.${feedId}.${year}`

beforeEach(() => {
  localStorage.clear()
  vi.spyOn(api, "years").mockImplementation(async (feedId) => [
    { feed_id: feedId, year: 2026, active: true },
    { feed_id: feedId, year: 2025, active: false },
  ])
  vi.spyOn(api, "months").mockResolvedValue([month])
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
    expect(screen.getByRole("button", { name: "Refresh year" })).toBeVisible()
    expect(screen.getByRole("button", { name: "Restore year" })).toBeVisible()
    first.focus()
    await user.keyboard("{Enter}")
    expect(screen.getByRole("button", { name: "Collapse 2026" })).toHaveAttribute("aria-expanded", "true")
    expect(document.getElementById(first.getAttribute("aria-controls")!)).toBeVisible()
    await user.click(screen.getByRole("button", { name: "Expand 2025" }))
    expect(screen.getAllByRole("textbox", { name: "Public folder link" })).toHaveLength(2)
    await user.click(screen.getByRole("button", { name: "Collapse 2026" }))
    expect(screen.getByRole("button", { name: "Collapse 2025" })).toHaveAttribute("aria-expanded", "true")
    expect(screen.getAllByRole("textbox", { name: "Public folder link" })).toHaveLength(1)
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
