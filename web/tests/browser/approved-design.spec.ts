import { expect, test } from "@playwright/test"

const feed = { id: "design-feed", display_name: "Operator research notes with a long author name", description: "Source folders and publication decisions", active: true, refresh_interval_seconds: null, extraction_engine: null }
const month = { id: "design-month", feed_id: feed.id, year: 2026, month: 11, provider: "fixture", source_url: `https://disk.yandex.ru/${"long-source-path/".repeat(8)}`, active: true, last_attempt_at: null, next_refresh_at: null, last_result: null, last_error: null }

for (const viewport of [{ width: 320, height: 740 }, { width: 375, height: 812 }, { width: 768, height: 1024 }, { width: 1024, height: 768 }, { width: 1440, height: 900 }, { width: 812, height: 375 }]) {
  test(`approved screens reflow at ${viewport.width}x${viewport.height}`, async ({ page }) => {
    await page.setViewportSize(viewport)
    await page.emulateMedia({ reducedMotion: "reduce" })
    await page.route("**/api/management/**", async (route) => {
      const path = new URL(route.request().url()).pathname
      const body = path.endsWith("/session") ? { status: "authenticated" }
        : path.endsWith("/overview") ? { source_folder_count: 1, needs_review_count: 0 }
        : path.endsWith("/feeds") ? [feed]
        : path.endsWith("/years") ? [{ feed_id: feed.id, year: 2026, active: true }]
        : path.endsWith("/months") ? [month]
        : path.endsWith("/documents") ? [{ id: "design-doc", filename: `${"long-filename-".repeat(12)}.pdf`, promotional_classification: "promotional", promotional_override: "include", extraction_status: "failure", last_error: "Extraction failed. Retry this document." }] : null
      await route.fulfill({ status: 200, json: body })
    })
    const noOverflow = async () => expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth)).toBe(true)
    await page.goto("/manage/")
    await expect(page.getByText("No documents need attention")).toBeVisible()
    await expect(page.getByRole("button", { name: "Sign out" })).toBeVisible()
    await noOverflow()
    await page.goto("/manage/feeds")
    await expect(page.getByRole("tab", { name: "Active (1)" })).toBeVisible()
    await expect(page.getByRole("button", { name: "Add year" })).toHaveCount(0)
    await noOverflow()
    await page.goto("/manage/settings")
    await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible()
    await expect(page.getByRole("heading", { name: "Routing precedence" })).toBeVisible()
    await expect(page.getByLabel("Proxy URL")).toBeVisible()
    await page.getByLabel("Proxy URL").focus()
    await page.keyboard.press("Tab")
    await page.keyboard.press("Shift+Tab")
    await expect(page.getByLabel("Proxy URL")).toBeFocused()
    expect(await page.getByLabel("Proxy URL").evaluate((element) => getComputedStyle(element).boxShadow)).not.toBe("none")
    await expect(page.getByLabel("Proxy URL")).toHaveCSS("animation-duration", "0s")
    await noOverflow()
    await page.goto("/manage/feeds")
    await page.getByRole("button", { name: /Operator research notes/ }).click()
    await expect(page.getByRole("heading", { name: feed.display_name })).toBeVisible()
    const yearToggle = page.getByRole("button", { name: "Expand 2026" })
    await yearToggle.focus()
    await page.keyboard.press("Enter")
    await expect(page.getByRole("button", { name: "Collapse 2026" })).toHaveAttribute("aria-expanded", "true")
    const sourceDetails = page.getByRole("button", { name: "Show source details for November" })
    await sourceDetails.focus()
    await page.keyboard.press("Enter")
    await expect(page.getByRole("button", { name: "Hide source details for November" })).toHaveAttribute("aria-expanded", "true")
    await expect(page.getByRole("link", { name: month.source_url })).toBeVisible()
    await expect(page.getByRole("region", { name: "November refresh status" })).toBeVisible()
    await expect(page.getByRole("button", { name: "Fetch and parse" })).toBeVisible()
    await expect(page.getByRole("button", { name: "Refresh now" })).toBeVisible()
    await expect(page.getByRole("button", { name: "Delete permanently" })).toBeVisible()
    await noOverflow()
    const addYear = page.getByRole("region", { name: "Feed details" }).getByRole("button", { name: "Add year" })
    await addYear.click()
    await expect(page.getByRole("dialog", { name: "Add year" })).toBeVisible()
    await page.keyboard.press("Escape")
    await expect(page.getByRole("dialog", { name: "Add year" })).toBeHidden()
    await expect(addYear).toBeFocused()
    await page.goto("/manage/feeds/new")
    await page.getByRole("button", { name: "Create feed" }).click()
    await expect(page.getByTestId("feed-error-summary")).toBeFocused()
    await page.getByTestId("feed-error-summary").getByRole("link").click()
    await expect(page.getByLabel("Author name")).toBeFocused()
    await noOverflow()
    await page.goto(`/manage/pdf-review/${feed.id}/2026/${month.id}`)
    await expect(page.getByRole("combobox", { name: /Publication decision for/ })).toHaveValue("include")
    await expect(page.locator("[data-slot=badge]").getByText("Promotional", { exact: true })).toBeVisible()
    await expect(page.getByText(/Successful extraction is required/)).toBeVisible()
    await noOverflow()
  })
}
