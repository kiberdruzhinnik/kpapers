import { expect, test } from "@playwright/test"

const cases = [
  { name: "Moscow local time", timezoneId: "Europe/Moscow", last: "2026-11-08T18:09:15Z", next: "2026-11-08T21:01:02Z", expectedLast: "2026-11-08 21:09:15", expectedNext: "2026-11-09 00:01:02" },
  { name: "New York daylight-saving transition", timezoneId: "America/New_York", last: "2026-03-08T06:59:59Z", next: "2026-03-08T07:00:00Z", expectedLast: "2026-03-08 01:59:59", expectedNext: "2026-03-08 03:00:00" },
  { name: "New York winter and summer offsets", timezoneId: "America/New_York", last: "2026-01-08T18:09:15Z", next: "2026-07-08T18:09:15Z", expectedLast: "2026-01-08 13:09:15", expectedNext: "2026-07-08 14:09:15" },
]

for (const scenario of cases) {
  test.describe(scenario.name, () => {
    test.use({ timezoneId: scenario.timezoneId })
    test("renders both refresh instants in browser local time", async ({ page }) => {
      const month = {
        id: "timezone-month", feed_id: "timezone-feed", year: 2026, month: 11, provider: "fixture", source_url: "fixture://source/",
        active: true, last_attempt_at: scenario.last, next_refresh_at: scenario.next, last_result: "success", last_error: null,
      }
      await page.route("**/api/management/**", async (route) => {
        const path = new URL(route.request().url()).pathname
        const body = path.endsWith("/session") ? { status: "authenticated" }
          : path.endsWith("/feeds") ? [{ id: "timezone-feed", display_name: "Timezone author", description: "", active: true, refresh_interval_seconds: null }]
          : path.endsWith("/years") ? [{ feed_id: "timezone-feed", year: 2026, active: true }]
          : path.endsWith("/months") ? [month] : null
        await route.fulfill({ status: 200, json: body })
      })
      await page.goto("/manage/feeds/timezone-feed")
      await page.getByRole("button", { name: "Expand 2026" }).click()
      await page.getByRole("button", { name: "Manage source" }).click()
      const card = page.getByRole("heading", { name: "November" }).locator("xpath=ancestor::*[@data-slot='card'][1]")
      await expect(card.getByText(scenario.expectedLast, { exact: true })).toBeVisible()
      await expect(card.getByText(scenario.expectedNext, { exact: true })).toBeVisible()
      expect(month.last_attempt_at).toBe(scenario.last)
      expect(month.next_refresh_at).toBe(scenario.next)
    })
  })
}
