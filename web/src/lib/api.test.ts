import { describe, expect, it, vi } from "vitest"
import { ApiError, api, request } from "./api"

describe("management API request", () => {
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
})
