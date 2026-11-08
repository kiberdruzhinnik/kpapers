import { render, screen, waitFor, within } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it, vi } from "vitest"
import App from "./App"

const sampleFeed = {
  id: "feed-1",
  display_name: "Ada Lovelace",
  description: "Research notes",
  active: true,
  refresh_interval_seconds: null,
}

function response(body: unknown, status = 200) {
  return new Response(body === null ? null : JSON.stringify(body), {
    status,
    headers: body === null ? {} : { "Content-Type": "application/json" },
  })
}

function installApi(initiallySignedIn = false) {
  let signedIn = initiallySignedIn
  const fetchMock = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
    const path = String(input)
    if (path === "/api/management/session" && init?.method === "DELETE") {
      signedIn = false
      return response(null, 204)
    }
    if (path === "/api/management/session") return signedIn ? response({ status: "authenticated" }) : response(null, 401)
    if (path === "/api/management/login") {
      const credentials = JSON.parse(String(init?.body)) as { username: string; password: string }
      if (credentials.username !== "operator" || credentials.password !== "correct horse") return response(null, 401)
      signedIn = true
      return response({ authenticated: true })
    }
    if (path === "/api/management/feeds") return signedIn ? response([sampleFeed]) : response(null, 401)
    return response(null, 404)
  })
  vi.stubGlobal("fetch", fetchMock)
  return fetchMock
}

describe("operator session and dashboard shell", () => {
  it("restores a signed-in session and displays the workspace", async () => {
    installApi(true)

    render(<App />)

    expect(await screen.findByRole("heading", { name: "Overview" })).toBeInTheDocument()
    expect(screen.getByText("Active feeds")).toBeInTheDocument()
    expect(screen.getByRole("button", { name: "Create feed" })).toBeInTheDocument()
  })

  it("keeps management private and supports keyboard-accessible sign-in feedback", async () => {
    installApi(false)
    const user = userEvent.setup()

    render(<App />)

    const username = await screen.findByRole("textbox", { name: "Username" })
    expect(username).toHaveFocus()
    await user.tab()
    expect(screen.getByLabelText("Password")).toHaveFocus()
    await user.tab()
    expect(screen.getByRole("button", { name: "Sign in" })).toHaveFocus()
    expect(screen.queryByRole("navigation", { name: "Primary navigation" })).not.toBeInTheDocument()

    await user.type(username, "operator")
    await user.type(screen.getByLabelText("Password"), "wrong password")
    await user.click(screen.getByRole("button", { name: "Sign in" }))
    expect(await screen.findByRole("alert")).toHaveTextContent("The username or password is incorrect.")

    await user.clear(screen.getByLabelText("Password"))
    await user.type(screen.getByLabelText("Password"), "correct horse")
    await user.click(screen.getByRole("button", { name: "Sign in" }))
    expect(await screen.findByRole("heading", { name: "Overview" })).toBeInTheDocument()
  })

  it("navigates between dashboard sections and signs out", async () => {
    const fetchMock = installApi(true)
    const user = userEvent.setup()

    render(<App />)

    await screen.findByRole("heading", { name: "Overview" })
    const nav = screen.getByRole("navigation", { name: "Primary navigation" })
    await user.click(within(nav).getByRole("button", { name: /Feeds/ }))
    expect(await screen.findByRole("heading", { name: "Feeds" })).toBeInTheDocument()
    await user.click(screen.getByRole("button", { name: /PDF review/ }))
    expect(await screen.findByRole("heading", { name: "PDF review" })).toBeInTheDocument()

    await user.click(screen.getByRole("button", { name: /Sign out/ }))
    expect(await screen.findByRole("heading", { name: "Welcome back" })).toBeInTheDocument()
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith("/api/management/session", expect.objectContaining({ method: "DELETE" })))
  })
})
