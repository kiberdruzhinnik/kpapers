import { expect, test } from "@playwright/test"

test.use({ viewport: { width: 375, height: 812 } })

test("phone layout keeps navigation, management validation, and keyboard sign-in usable", async ({ page }) => {
  await page.goto("/")
  await expect(page.getByLabel("Username")).toBeFocused()
  await page.keyboard.type(process.env.OPERATOR_USERNAME ?? "operator")
  await page.keyboard.press("Tab")
  await page.keyboard.type(process.env.OPERATOR_PASSWORD ?? "")
  await page.keyboard.press("Tab")
  await page.keyboard.press("Enter")
  await expect(page.getByRole("heading", { name: "Overview" })).toBeVisible()

  await page.getByRole("button", { name: "Open navigation" }).click()
  const mobileNav = page.getByRole("navigation", { name: "Primary navigation" })
  await mobileNav.getByRole("button", { name: /Feeds/ }).focus()
  await page.keyboard.press("Enter")
  await expect(page.getByRole("heading", { name: "Feeds", exact: true })).toBeVisible()
  await expect(page.getByRole("navigation", { name: "Primary navigation" })).toBeHidden()

  await page.getByRole("button", { name: "Create feed" }).click()
  await expect(page).toHaveURL(/\/manage\/feeds\/new$/)
  await page.getByLabel("Author name").fill("Mobile validation feed")
  await page.getByLabel("Refresh interval (optional)").fill("0")
  await page.getByRole("button", { name: "Create feed" }).click()
  await expect(page.getByTestId("feed-error-summary")).toContainText("Refresh interval must be greater than zero hours.")

  const dimensions = await page.evaluate(() => ({
    viewport: document.documentElement.clientWidth,
    content: document.documentElement.scrollWidth,
  }))
  expect(dimensions.content).toBeLessThanOrEqual(dimensions.viewport)
  await expect(page.getByRole("button", { name: "Open navigation" })).toBeVisible()
})
