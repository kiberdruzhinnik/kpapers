import { useEffect, useState, type FormEvent } from "react"
import { toast } from "sonner"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Field, FieldDescription, FieldError, FieldGroup, FieldLabel } from "@/components/ui/field"
import { Input } from "@/components/ui/input"
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { api, type FeedProxySettings } from "@/lib/api"

function validateProxyUrl(value: string): string | null {
  let url: URL
  try {
    url = new URL(value)
  } catch {
    return "Enter a complete proxy URL."
  }
  if (url.protocol !== "http:" && url.protocol !== "socks5:") return "Proxy URL must use http:// or socks5://."
  if (!url.hostname || !url.port) return "Include a proxy host and port."
  if (url.search || url.hash) return "Proxy URL must not include a query or fragment."
  return null
}

export function FeedProxySettings({ feedId }: { feedId: string }) {
  const [settings, setSettings] = useState<FeedProxySettings | null>(null)
  const [mode, setMode] = useState<FeedProxySettings["mode"]>("inherit")
  const [proxyUrl, setProxyUrl] = useState("")
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)
  const [pending, setPending] = useState(false)

  useEffect(() => {
    let active = true
    api.feedProxy(feedId).then((result) => {
      if (!active) return
      setSettings(result)
      setMode(result.mode)
    }).catch((cause) => {
      if (active) setError(cause instanceof Error ? cause.message : "Could not load this feed's proxy settings.")
    }).finally(() => {
      if (active) setLoading(false)
    })
    return () => { active = false }
  }, [feedId])

  async function save(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (mode === "custom") {
      const validationError = validateProxyUrl(proxyUrl.trim())
      if (!proxyUrl.trim()) {
        setError("Enter the custom proxy URL.")
        return
      }
      if (validationError) {
        setError(validationError)
        return
      }
    }
    setPending(true)
    setError(null)
    try {
      const result = await api.saveFeedProxy(feedId, mode, mode === "custom" ? proxyUrl.trim() : undefined)
      setSettings(result)
      setProxyUrl("")
      toast.success("Feed proxy settings saved")
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Could not save this feed's proxy settings.")
    } finally {
      setPending(false)
    }
  }

  const customConfigured = settings?.mode === "custom" && settings.configured

  return (
    <Card>
      <CardHeader>
        <CardTitle>Fetch proxy</CardTitle>
        <CardDescription>Choose the network route for discovering PDFs and fetching article files for this feed.</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-5">
        {loading && <p role="status" className="text-sm text-muted-foreground">Loading feed proxy settings…</p>}
        {error && <Alert data-testid="feed-proxy-error" variant="destructive"><AlertTitle>Proxy settings could not be saved</AlertTitle><AlertDescription>{error}</AlertDescription></Alert>}
        {customConfigured && <p className="break-all rounded-lg border bg-muted/30 px-3 py-2 text-xs text-muted-foreground" data-testid="feed-proxy-status">Custom proxy configured: {settings.endpoint ?? "endpoint hidden"}. Credentials are not shown again.</p>}
        <form className="flex flex-col gap-5" onSubmit={(event) => void save(event)}>
          <FieldGroup>
            <Field>
              <FieldLabel htmlFor={`feed-proxy-mode-${feedId}`}>Proxy mode</FieldLabel>
              <Select value={mode} onValueChange={(value) => { setMode(value as FeedProxySettings["mode"]); setError(null) }}>
                <SelectTrigger aria-label="Proxy mode" id={`feed-proxy-mode-${feedId}`}>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    <SelectItem value="inherit">Inherit global settings</SelectItem>
                    <SelectItem value="direct">Connect directly</SelectItem>
                    <SelectItem value="custom">Use a custom proxy</SelectItem>
                  </SelectGroup>
                </SelectContent>
              </Select>
              <FieldDescription>{mode === "inherit" ? "Uses the global GUI proxy, then KPAPERS_PROXY_URL, then a direct connection." : mode === "direct" ? "Bypasses both the global GUI proxy and KPAPERS_PROXY_URL." : "Uses this feed's HTTP or SOCKS5 proxy for discovery and downloads."}</FieldDescription>
            </Field>
            {mode === "custom" && <Field data-invalid={Boolean(error && !proxyUrl.trim())}>
              <FieldLabel htmlFor={`feed-proxy-url-${feedId}`}>Custom proxy URL</FieldLabel>
              <Input
                aria-describedby={`feed-proxy-help-${feedId}`}
                aria-invalid={Boolean(error && !proxyUrl.trim())}
                autoComplete="off"
                id={`feed-proxy-url-${feedId}`}
                onChange={(event) => { setProxyUrl(event.target.value); setError(null) }}
                placeholder={customConfigured ? "Enter a replacement URL" : "socks5://user:password@proxy.example:1080"}
                spellCheck={false}
                type="password"
                value={proxyUrl}
              />
              <FieldDescription id={`feed-proxy-help-${feedId}`}>{customConfigured ? "The saved URL is write-only. Leave this field empty to keep the current custom proxy unchanged." : "Use http:// or socks5:// with a host and port. Credentials are not returned to the browser."}</FieldDescription>
              {error && mode === "custom" && <FieldError>{error}</FieldError>}
            </Field>}
          </FieldGroup>
          <div className="flex justify-end">
            <Button disabled={pending || loading || (mode === "custom" && customConfigured && !proxyUrl.trim() && settings?.mode === mode)} type="submit">{pending ? "Saving…" : "Save proxy settings"}</Button>
          </div>
        </form>
      </CardContent>
    </Card>
  )
}
