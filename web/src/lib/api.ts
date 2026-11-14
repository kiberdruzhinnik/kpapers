export type Feed = {
  id: string
  display_name: string
  description: string
  active: boolean
  refresh_interval_seconds: number | null
}

export type Year = {
  feed_id: string
  year: number
  active: boolean
}

export type Month = {
  id: string
  feed_id: string
  year: number
  month: number
  provider: string
  source_url: string
  active: boolean
  last_attempt_at: string | null
  next_refresh_at: string | null
  last_result: string | null
  last_error: string | null
}

export type SourceDocument = {
  id: string
  filename: string
  source_url: string
  promotional_classification: string
  promotional_override: "include" | "exclude" | null
  extraction_status: string
  last_error: string | null
}

export type ReviewQueueMode = "needs-review" | "archived"
export type ReviewQueueDocument = SourceDocument & {
  feed_id: string
  feed_name: string
  year: number
  month: number
  month_id: string
  archived: boolean
}

export type QueuedRefresh = { job_id: string; status: string }
export type OverviewSummary = { source_folder_count: number; needs_review_count: number }
export type DiscoveryResult = { discovered_pdfs: number }
export type ExtractionResult = {
  status: string
  page_count: number
  characters: number
  first_page_sample: string
}

export class ApiError extends Error {
  readonly status: number

  constructor(status: number, message: string) {
    super(message)
    this.name = "ApiError"
    this.status = status
  }
}

export async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  const headers = new Headers(init.headers)
  if (init.body !== undefined && !headers.has("Content-Type")) {
    headers.set("Content-Type", "application/json")
  }

  let response: Response
  try {
    response = await fetch(path, {
      ...init,
      headers,
      credentials: "same-origin",
    })
  } catch {
    throw new ApiError(0, "Unable to reach Kpapers. Check your connection and try again.")
  }

  if (!response.ok) {
    const message = response.status === 401
      ? "Your session has expired. Please sign in again."
      : response.status === 400
        ? "Please check the information and try again."
        : response.status === 404
          ? "This item could not be found. It may have been removed."
          : response.status >= 500
            ? "Kpapers could not complete that request. Try again shortly."
            : `Request failed (${response.status}).`
    throw new ApiError(response.status, message)
  }

  if (response.status === 204) return undefined as T
  const contentType = response.headers.get("Content-Type") ?? ""
  if (contentType.includes("application/json")) return await response.json() as T
  return undefined as T
}

const json = (method: string, payload?: unknown): RequestInit => ({
  method,
  ...(payload === undefined ? {} : { body: JSON.stringify(payload) }),
})

export const api = {
  session: () => request<{ status: string }>("/api/management/session"),
  login: (username: string, password: string) => request<{ authenticated: boolean }>(
    "/api/management/login",
    json("POST", { username, password }),
  ),
  logout: () => request<void>("/api/management/session", json("DELETE")),
  feeds: () => request<Feed[]>("/api/management/feeds"),
  overview: () => request<OverviewSummary>("/api/management/overview"),
  reviewQueue: (archived: boolean) => request<ReviewQueueDocument[]>(
    `/api/management/review-queue?archived=${archived}`,
  ),
  createFeed: (payload: Pick<Feed, "display_name" | "description"> & { refresh_interval_seconds: number | null }) =>
    request<Feed>("/api/management/feeds", json("POST", payload)),
  updateFeed: (id: string, payload: Pick<Feed, "display_name" | "description"> & { refresh_interval_seconds: number | null }) =>
    request<void>(`/api/management/feeds/${encodeURIComponent(id)}`, json("PUT", payload)),
  archiveFeed: (id: string) => request<void>(`/api/management/feeds/${encodeURIComponent(id)}/archive`, json("POST")),
  restoreFeed: (id: string) => request<void>(`/api/management/feeds/${encodeURIComponent(id)}/restore`, json("POST")),
  deleteFeed: (id: string) => request<void>(`/api/management/feeds/${encodeURIComponent(id)}/permanent`, json("DELETE")),
  refreshFeed: (id: string) => request<QueuedRefresh>(`/api/management/feeds/${encodeURIComponent(id)}/refresh`, json("POST")),
  years: (feedId: string) => request<Year[]>(`/api/management/feeds/${encodeURIComponent(feedId)}/years`),
  createYear: (feedId: string, year: number) => request<Year>(
    `/api/management/feeds/${encodeURIComponent(feedId)}/years`, json("POST", { year }),
  ),
  deleteYear: (feedId: string, year: number) => request<void>(
    `/api/management/feeds/${encodeURIComponent(feedId)}/years/${year}`, json("DELETE"),
  ),
  archiveYear: (feedId: string, year: number) => request<void>(
    `/api/management/feeds/${encodeURIComponent(feedId)}/years/${year}/archive`, json("POST"),
  ),
  restoreYear: (feedId: string, year: number) => request<void>(
    `/api/management/feeds/${encodeURIComponent(feedId)}/years/${year}/restore`, json("POST"),
  ),
  refreshYear: (feedId: string, year: number) => request<QueuedRefresh>(
    `/api/management/feeds/${encodeURIComponent(feedId)}/years/${year}/refresh`, json("POST"),
  ),
  months: (feedId: string, year: number) => request<Month[]>(
    `/api/management/feeds/${encodeURIComponent(feedId)}/years/${year}/months`,
  ),
  createMonth: (feedId: string, year: number, month: number, source_url: string) => request<Month>(
    `/api/management/feeds/${encodeURIComponent(feedId)}/years/${year}/months/${month}`,
    json("PUT", { source_url }),
  ),
  documents: (monthId: string) => request<SourceDocument[]>(
    `/api/management/months/${encodeURIComponent(monthId)}/documents`,
  ),
  documentSourceUrl: (documentId: string) =>
    `/api/management/documents/${encodeURIComponent(documentId)}/source`,
  deleteMonth: (monthId: string) => request<void>(`/api/management/months/${encodeURIComponent(monthId)}`, json("DELETE")),
  archiveMonth: (monthId: string) => request<void>(`/api/management/months/${encodeURIComponent(monthId)}/archive`, json("POST")),
  restoreMonth: (monthId: string) => request<void>(`/api/management/months/${encodeURIComponent(monthId)}/restore`, json("POST")),
  discoverMonth: (monthId: string) => request<DiscoveryResult>(`/api/management/months/${encodeURIComponent(monthId)}/discover`, json("POST")),
  refreshMonth: (monthId: string) => request<QueuedRefresh>(`/api/management/months/${encodeURIComponent(monthId)}/refresh`, json("POST")),
  extractDocument: (id: string, force = false) => request<ExtractionResult>(
    `/api/management/documents/${encodeURIComponent(id)}/extract${force ? "?force=true" : ""}`,
    json("POST"),
  ),
  setPromotion: (id: string, override_value: "include" | "exclude" | null) => request<void>(
    `/api/management/documents/${encodeURIComponent(id)}/promotion`, json("PUT", { override_value }),
  ),
}
