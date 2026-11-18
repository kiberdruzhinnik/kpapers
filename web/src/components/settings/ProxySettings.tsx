import { useEffect, useState, type FormEvent } from "react"
import { Check, Trash2 } from "lucide-react"
import { toast } from "sonner"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Field, FieldDescription, FieldError, FieldGroup, FieldLabel } from "@/components/ui/field"
import { Input } from "@/components/ui/input"
import { api, type ProxySettings } from "@/lib/api"

function validateProxyUrl(value: string): string | null {
  let url: URL
  try {
    url = new URL(value)
  } catch {
    return "Enter a complete proxy URL."
  }
  if (url.protocol !== "http:" && url.protocol !== "socks5:") {
    return "Proxy URL must use http:// or socks5://."
  }
  if (!url.hostname || !url.port) return "Include a proxy host and port."
  if (url.search || url.hash) return "Proxy URL must not include a query or fragment."
  return null
}

export function ProxySettingsPage() {
  const [settings, setSettings] = useState<ProxySettings | null>(null)
  const [proxyUrl, setProxyUrl] = useState("")
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)
  const [pending, setPending] = useState(false)

  useEffect(() => {
    let active = true
    api.globalProxy().then((result) => {
      if (active) setSettings(result)
    }).catch((cause) => {
      if (active) setError(cause instanceof Error ? cause.message : "Could not load proxy settings.")
    }).finally(() => {
      if (active) setLoading(false)
    })
    return () => { active = false }
  }, [])

  async function save(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const validationError = validateProxyUrl(proxyUrl.trim())
    if (validationError) {
      setError(validationError)
      return
    }
    setPending(true)
    setError(null)
    try {
      const result = await api.saveGlobalProxy(proxyUrl.trim())
      setSettings(result)
      setProxyUrl("")
      toast.success("Global proxy saved", { description: "Feeds using Inherit will use this proxy." })
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Could not save proxy settings.")
    } finally {
      setPending(false)
    }
  }

  async function clear() {
    setPending(true)
    setError(null)
    try {
      await api.clearGlobalProxy()
      setSettings({ configured: false, endpoint: null })
      setProxyUrl("")
      toast.success("Global proxy cleared", { description: "Inherited feeds will use the environment fallback, if configured." })
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Could not clear proxy settings.")
    } finally {
      setPending(false)
    }
  }

  return (
    <div className="mx-auto flex w-full max-w-6xl flex-col gap-6 px-4 py-7 sm:px-6 sm:py-8 lg:px-8">
      <div className="max-w-3xl">
        <h1 className="text-[28px] font-semibold leading-8 tracking-tight">Settings</h1>
        <p className="mt-2 text-sm text-muted-foreground">Configure the network route used to discover PDFs and fetch article files.</p>
      </div>

      <div className="grid items-start gap-5 lg:grid-cols-[minmax(0,1.15fr)_minmax(280px,0.85fr)]">
      <Card className="min-w-0">
        <CardHeader>
          <CardTitle>Global fetch proxy</CardTitle>
          <CardDescription>Configure the network route used for PDF discovery and document retrieval.</CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-5">
          {loading ? <p role="status" className="text-sm text-muted-foreground">Loading proxy settings…</p> : settings?.configured ? (
            <div aria-live="polite" className="rounded-lg border bg-muted/30 px-3 py-3 text-sm" data-testid="global-proxy-status">
              <p className="font-medium">A global proxy is configured</p>
              <p className="mt-1 break-all font-mono text-xs text-muted-foreground">{settings.endpoint ?? "Configured proxy endpoint hidden"}</p>
              <p className="mt-1 text-xs text-muted-foreground">Credentials are not shown again. Enter a new URL below to replace the saved proxy.</p>
            </div>
          ) : !error && (
            <p className="rounded-lg border bg-muted/30 px-3 py-3 text-sm text-muted-foreground" data-testid="global-proxy-status">No global GUI proxy is configured.</p>
          )}

          {error && <Alert data-testid="proxy-settings-error" variant="destructive"><AlertTitle>Proxy settings could not be saved</AlertTitle><AlertDescription>{error}</AlertDescription></Alert>}

          <form className="flex flex-col gap-5" onSubmit={(event) => void save(event)}>
            <FieldGroup>
              <Field data-invalid={Boolean(error && !settings?.configured)}>
                <FieldLabel htmlFor="global-proxy-url">Proxy URL</FieldLabel>
                <Input
                  aria-describedby="global-proxy-help"
                  aria-invalid={Boolean(error && !settings?.configured)}
                  autoComplete="off"
                  id="global-proxy-url"
                  name="proxy_url"
                  onChange={(event) => { setProxyUrl(event.target.value); setError(null) }}
                  placeholder="http://user:password@proxy.example:8080"
                  spellCheck={false}
                  type="password"
                  value={proxyUrl}
                />
                <FieldDescription id="global-proxy-help">Use an HTTP URL or SOCKS5 URL such as socks5://user:password@proxy.example:1080. Saved credentials are never returned to the browser.</FieldDescription>
                {error && !settings?.configured && <FieldError>{error}</FieldError>}
              </Field>
            </FieldGroup>
            <div className="flex flex-col gap-2 sm:flex-row sm:justify-end">
              {settings?.configured && <Button disabled={pending || loading} onClick={() => void clear()} type="button" variant="outline"><Trash2 aria-hidden="true" data-icon="inline-start" />Clear global proxy</Button>}
              <Button disabled={pending || loading || !proxyUrl.trim()} type="submit">{pending ? "Saving…" : settings?.configured ? "Replace proxy" : <><Check aria-hidden="true" data-icon="inline-start" />Save proxy</>}</Button>
            </div>
          </form>
        </CardContent>
      </Card>
      <div className="flex flex-col gap-5">
        <Card>
          <CardHeader>
            <CardTitle>Routing precedence</CardTitle>
            <CardDescription>Feeds using Inherit follow this order.</CardDescription>
          </CardHeader>
          <CardContent>
            <ol className="flex flex-col gap-3 text-sm">
              <li className="flex gap-3"><span className="flex size-6 shrink-0 items-center justify-center rounded-full bg-muted text-xs font-semibold">1</span><span><span className="font-medium">Global GUI proxy</span><span className="mt-0.5 block text-muted-foreground">The proxy configured on this page.</span></span></li>
              <li className="flex gap-3"><span className="flex size-6 shrink-0 items-center justify-center rounded-full bg-muted text-xs font-semibold">2</span><span><span className="font-medium">Environment fallback</span><span className="mt-0.5 block text-muted-foreground"><code className="rounded bg-muted px-1 py-0.5 text-xs">KPAPERS_PROXY_URL</code>, when configured.</span></span></li>
              <li className="flex gap-3"><span className="flex size-6 shrink-0 items-center justify-center rounded-full bg-muted text-xs font-semibold">3</span><span><span className="font-medium">Direct connection</span><span className="mt-0.5 block text-muted-foreground">Used when no proxy is configured.</span></span></li>
            </ol>
            <p className="mt-4 border-t pt-4 text-sm text-muted-foreground">A feed can override inheritance by using its own proxy or connecting directly.</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Credential handling</CardTitle>
          </CardHeader>
          <CardContent className="text-sm text-muted-foreground">Saved proxy credentials are write-only. They are never returned to the browser or shown again. Enter a new URL to replace them.</CardContent>
        </Card>
      </div>
      </div>
    </div>
  )
}
