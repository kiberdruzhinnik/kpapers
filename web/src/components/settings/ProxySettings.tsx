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
    <div className="flex flex-col gap-6">
      <div>
        <h1>Settings</h1>
        <p className="mt-2 max-w-2xl text-sm text-muted-foreground">Configure the network route used when feeds discover PDFs and fetch article files.</p>
      </div>

      <Card className="max-w-3xl">
        <CardHeader>
          <CardTitle>Global fetch proxy</CardTitle>
          <CardDescription>Feeds set to Inherit use this proxy, then the deployment environment fallback, then a direct connection. A feed can instead use its own proxy or connect directly.</CardDescription>
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
    </div>
  )
}
