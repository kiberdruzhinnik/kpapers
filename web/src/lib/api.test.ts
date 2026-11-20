import { afterEach, describe, expect, it, vi } from "vitest"
import { ApiError, api, request } from "./api"

afterEach(() => {
  vi.unstubAllGlobals()
  vi.restoreAllMocks()
})

describe("management API request", () => {
  it("loads overview counts from the management summary endpoint", async () => {
    const summary = { source_folder_count: 3, needs_review_count: 2 }
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify(summary), {
      status: 200,
      headers: { "Content-Type": "application/json" },
    }))
    vi.stubGlobal("fetch", fetchMock)

    await expect(api.overview()).resolves.toEqual(summary)
    expect(fetchMock).toHaveBeenCalledWith("/api/management/overview", expect.objectContaining({ credentials: "same-origin" }))
  })

  it("loads active and archived global review queues", async () => {
    const fetchMock = vi.fn().mockImplementation(() => Promise.resolve(new Response("[]", {
      status: 200,
      headers: { "Content-Type": "application/json" },
    })))
    vi.stubGlobal("fetch", fetchMock)

    await expect(api.reviewQueue(false)).resolves.toEqual([])
    await expect(api.reviewQueue(true)).resolves.toEqual([])
    expect(fetchMock).toHaveBeenNthCalledWith(1, "/api/management/review-queue?archived=false", expect.objectContaining({ credentials: "same-origin" }))
    expect(fetchMock).toHaveBeenNthCalledWith(2, "/api/management/review-queue?archived=true", expect.objectContaining({ credentials: "same-origin" }))
  })

  it("downloads feed, year, and month archives with same-origin credentials", async () => {
    const createObjectURL = vi.fn().mockReturnValue("blob:export")
    const revokeObjectURL = vi.fn()
    vi.stubGlobal("URL", { createObjectURL, revokeObjectURL })
    const downloadNames: string[] = []
    const click = vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(function (this: HTMLAnchorElement) {
      downloadNames.push(this.download)
    })
    const fetchMock = vi.fn().mockImplementation(() => Promise.resolve(new Response("zip-bytes", {
      status: 200,
      headers: { "Content-Type": "application/zip" },
    })))
    vi.stubGlobal("fetch", fetchMock)

    await api.exportFeed("feed/1", "Ada Lovelace")
    await api.exportYear("feed/1", 2026, "Ada Lovelace")
    await api.exportMonth("month-1", "Ada Lovelace", 2026, 2)

    expect(fetchMock).toHaveBeenNthCalledWith(1, "/api/management/feeds/feed%2F1/export", { credentials: "same-origin" })
    expect(fetchMock).toHaveBeenNthCalledWith(2, "/api/management/feeds/feed%2F1/years/2026/export", { credentials: "same-origin" })
    expect(fetchMock).toHaveBeenNthCalledWith(3, "/api/management/months/month-1/export", { credentials: "same-origin" })
    expect(click).toHaveBeenCalledTimes(3)
    expect(createObjectURL).toHaveBeenCalledTimes(3)
    expect(downloadNames).toEqual(["Ada-Lovelace.zip", "Ada-Lovelace-2026.zip", "Ada-Lovelace-2026-02.zip"])
  })

  it("reports export authentication failures without downloading a file", async () => {
    const click = vi.spyOn(HTMLAnchorElement.prototype, "click")
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(new Response(null, { status: 401 })))

    await expect(api.exportFeed("feed-1", "Ada")).rejects.toMatchObject({
      status: 401,
      message: "Your session has expired. Please sign in again.",
    })
    expect(click).not.toHaveBeenCalled()
  })

  it("sends same-origin credentials and JSON payloads", async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response('{"id":"feed-1"}', {
      status: 201,
      headers: { "Content-Type": "application/json" },
    }))
    vi.stubGlobal("fetch", fetchMock)

    await api.createFeed({ display_name: "Ada", description: "Notes", refresh_interval_seconds: null })

    expect(fetchMock).toHaveBeenCalledWith("/api/management/feeds", expect.objectContaining({
      method: "POST",
      credentials: "same-origin",
      body: JSON.stringify({ display_name: "Ada", description: "Notes", refresh_interval_seconds: null }),
    }))
    expect(fetchMock.mock.calls[0][1]?.headers).toBeInstanceOf(Headers)
    expect(new Headers(fetchMock.mock.calls[0][1]?.headers).get("Content-Type")).toBe("application/json")
  })

  it("returns an empty result for no-content responses", async () => {
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(new Response(null, { status: 204 })))
    await expect(api.logout()).resolves.toBeUndefined()
  })

  it("normalizes unauthorized, validation, and server failures", async () => {
    vi.stubGlobal("fetch", vi.fn()
      .mockResolvedValueOnce(new Response(null, { status: 401 }))
      .mockResolvedValueOnce(new Response(null, { status: 400 }))
      .mockResolvedValueOnce(new Response(null, { status: 503 })))

    await expect(request("/private")).rejects.toMatchObject({
      name: "ApiError", status: 401, message: "Your session has expired. Please sign in again.",
    })
    await expect(request("/form")).rejects.toMatchObject({ status: 400, message: "Please check the information and try again." })
    await expect(request("/service")).rejects.toMatchObject({ status: 503, message: "Kpapers could not complete that request. Try again shortly." })
  })

  it("wraps network errors in a user-readable API error", async () => {
    vi.stubGlobal("fetch", vi.fn().mockRejectedValue(new TypeError("offline")))
    const pending = request("/offline")
    await expect(pending).rejects.toBeInstanceOf(ApiError)
    await expect(pending).rejects.toMatchObject({
      status: 0, message: "Unable to reach Kpapers. Check your connection and try again.",
    })
  })

  it("surfaces malformed successful JSON responses", async () => {
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(new Response("{bad json", {
      status: 200,
      headers: { "Content-Type": "application/json" },
    })))

    await expect(request("/malformed-response")).rejects.toBeInstanceOf(SyntaxError)
  })
})
