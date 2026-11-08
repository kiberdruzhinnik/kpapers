import { expect, test } from "@playwright/test"

test("disposable stack serves the dashboard and operator sign-in", async ({ page }) => {
  await page.goto("/")
  await expect(page.getByRole("heading", { name: "Welcome back" })).toBeVisible()
  await expect(page.getByRole("button", { name: "Sign in" })).toBeEnabled()
})
