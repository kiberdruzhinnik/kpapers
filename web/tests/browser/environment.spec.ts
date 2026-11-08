import { expect, test } from "@playwright/test"

test("disposable stack serves the dashboard and operator sign-in", async ({ page }) => {
  await page.goto("/")
  await expect(page.getByRole("heading", { name: "Welcome back" })).toBeVisible()
  await expect(page.getByRole("button", { name: "Sign in" })).toBeEnabled()
})


test("Overview and review management paths load and reload through the web server", async ({ page }) => {
  await page.goto("/manage/")
  await page.getByLabel("Username").fill(process.env.OPERATOR_USERNAME ?? "operator")
  await page.getByLabel("Password").fill(process.env.OPERATOR_PASSWORD ?? "")
  await page.getByRole("button", { name: "Sign in" }).click()
  await expect(page.getByRole("heading", { name: "Overview" })).toBeVisible()
  await page.reload()
  await expect(page.getByRole("heading", { name: "Overview" })).toBeVisible()
  await page.goto("/manage/pdf-review")
  await expect(page.getByRole("heading", { name: "PDF review" })).toBeVisible()
  await page.reload()
  await expect(page.getByRole("heading", { name: "PDF review" })).toBeVisible()
})
