import { expect, test } from "@playwright/test"

const referenceTitleFragment = "Правдоустойчивый диван генерирует победу над ЕГЭ и коррупцией"

test("Miniflux renders the PDF's inline images in the article view", async ({ page }) => {
  const minifluxUrl = process.env.MINIFLUX_BASE_URL
  test.skip(!minifluxUrl, "This browser check runs with the Miniflux E2E stack")
  const baseURL = minifluxUrl!.replace(/\/$/, "")
  const username = process.env.MINIFLUX_USERNAME ?? "e2e"
  const password = process.env.MINIFLUX_PASSWORD ?? ""
  const authorization = `Basic ${Buffer.from(`${username}:${password}`).toString("base64")}`

  const deadline = Date.now() + 90_000
  let referenceEntry: { id: number; title: string; content: string } | undefined
  while (!referenceEntry && Date.now() < deadline) {
    const feedsResponse = await page.request.get(`${baseURL}/v1/feeds`, {
      headers: { Authorization: authorization },
    })
    if (feedsResponse.ok()) {
      const feedsPayload = await feedsResponse.json() as { id: number }[] | { feeds: { id: number }[] }
      const feeds = Array.isArray(feedsPayload) ? feedsPayload : feedsPayload.feeds
      for (const feed of feeds) {
        const entriesResponse = await page.request.get(
          `${baseURL}/v1/feeds/${feed.id}/entries?limit=100&order=id&direction=asc`,
          { headers: { Authorization: authorization } },
        )
        if (!entriesResponse.ok()) continue
        const entriesPayload = await entriesResponse.json() as
          | { id: number; title: string; content: string }[]
          | { entries: { id: number; title: string; content: string }[] }
        const entries = Array.isArray(entriesPayload) ? entriesPayload : entriesPayload.entries
        referenceEntry = entries.find((entry) => entry.title.includes(referenceTitleFragment))
        if (referenceEntry) break
      }
    }
    if (!referenceEntry) await page.waitForTimeout(1_000)
  }

  expect(referenceEntry, "Miniflux should contain the reference PDF article").toBeDefined()
  expect(referenceEntry!.content.match(/<img\s/g) ?? []).toHaveLength(2)

  await page.goto(baseURL)
  await page.locator('input[name="username"]').fill(username)
  await page.locator('input[name="password"]').fill(password)
  await page.locator('form button[type="submit"], form input[type="submit"]').first().click()
  await expect(page).not.toHaveURL(/\/login(?:\?|$)/)

  let entryHref: string | null = null
  for (const section of ["unread", "history"]) {
    await page.goto(`${baseURL}/${section}`)
    const entryLink = page.locator("a").filter({ hasText: referenceEntry!.title }).first()
    if (await entryLink.count()) {
      entryHref = await entryLink.getAttribute("href")
      break
    }
  }
  expect(entryHref, "Miniflux should link to the reference article from its entry list").toBeTruthy()
  await page.goto(new URL(entryHref!, baseURL).toString())
  await expect(page.locator("body")).toContainText(referenceEntry!.title)
  const images = page.locator('img[src^="data:image/jpeg;base64,"]')
  await expect(images).toHaveCount(2)

  for (let index = 0; index < 2; index += 1) {
    const image = images.nth(index)
    await image.scrollIntoViewIfNeeded()
    await expect(image).toBeVisible()
    await expect.poll(async () => image.evaluate((element) => {
      const img = element as HTMLImageElement
      const bounds = img.getBoundingClientRect()
      return img.complete && img.naturalWidth > 0 && bounds.width > 0 && bounds.height > 0
    }), { message: `inline image ${index + 1} should decode and occupy visible layout space` }).toBe(true)
  }
})
