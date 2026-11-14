# Marker extraction worker

This directory contains the optional private worker that lets a feed use
[Marker](https://github.com/datalab-to/marker) instead of Pdfium. The Rust API
remains the only public service. The worker is reachable only from the API on
an internal Compose network and has no published host port.

## Contract

`POST /extract` accepts `Content-Type: application/pdf` and a bounded body.
It returns a page-aware JSON document:

```json
{
  "page_count": 2,
  "pages": [
    {
      "lines": [
        {
          "runs": [{"text": "Title", "bold": true, "italic": false}],
          "size": 24.0,
          "gap_before": 0.0,
          "left": 50.0
        }
      ]
    }
  ]
}
```

Marker's block output is reduced to plain text and emphasis runs. The worker
never returns Marker HTML, links, images, attributes or scripts. When fast-mode
Marker merges an indented region into a single paragraph, the worker restores
line and paragraph boundaries and mixed emphasis from the PDF text layer before
returning the response. Marker 2.2.0 can leave an empty page's child list as
`None`; the worker normalises it before Marker's processors run so an ordinary
empty or page-number-only page does not fail the document. The Rust API
validates the page count, coordinates and sizes, then applies the existing
boundary-advertisement, printed-page-label, Sponsr, classification, title
recovery and safe-HTML pipeline.

`GET /healthz` returns `200` with `{"status":"ok","ready":true}` after models
are loaded, and `503` while starting.

## Resource and failure behavior

- Input size is limited by `MARKER_WORKER_MAX_INPUT_BYTES` (25 MiB by default,
  matching `MAX_PDF_BYTES`).
- Output size is limited by `MARKER_WORKER_MAX_RESPONSE_BYTES` (8 MiB).
- Concurrent conversions are limited by `MARKER_WORKER_MAX_CONCURRENCY`
  (1 by default). Extra requests receive `503`; they are not queued.
- A conversion exceeding `MARKER_WORKER_EXECUTION_TIMEOUT_SECONDS` (110 s by
  default, slightly below the API's 120 s request timeout) returns `504` and
  schedules a process exit. The container supervisor restarts the worker so
  the stuck conversion releases its capacity, even when the client has already
  disconnected. The bundled Compose service uses `restart: unless-stopped`.
- A conversion failure returns `422`; an unexpected failure returns a generic
  `500` without document text.
- Request logs contain only the HTTP request line. Document contents and
  conversion error details are not logged.

The API does not fall back to Pdfium when the worker is unavailable or returns
an error. The document's previous successful cached content remains available.

## Pinned runtime and licenses

| Component | Pin | License |
| --- | --- | --- |
| Python | 3.12 (`python:3.12-slim-bookworm`) | PSF |
| `marker-pdf` | 2.2.0 | Apache-2.0 |
| `surya-ocr` | 0.23.0 | Apache-2.0 |
| `indra-ocr` | 1.1.2 | Apache-2.0 |
| `pdftext` | 0.7.1 | Apache-2.0 |
| `torch` / `torchvision` | 2.7.0 / 0.22.0 CPU | BSD-3-Clause |
| Marker model weights | downloaded by Marker at first startup | Modified AI Pubs OpenRAIL-M |

Marker's code is Apache-2.0, but its model weights use the modified AI Pubs
OpenRAIL-M license. Review the current terms before commercial deployment.
Model files are downloaded into the `marker_models` volume on first startup.
Provision the volume in advance if the deployment host has restricted egress.
The default `MARKER_MODE=fast` is CPU-only. The worker uses Marker's text-layer
path only when every page has native text and no page contains an embedded image
or vector path. Without OCR the worker cannot prove an image contains no text,
and vector paths may contain outlined lettering missing from the text layer.
Either object selects a dedicated `fast` converter with `force_ocr` enabled. A
document with any image-only page also uses forced OCR,
so scanned content is not omitted by Marker's automatic text-layer decision.
Forms at Pdfium's object traversal limit are rejected because their contents
cannot be inspected safely. Explicitly setting `MARKER_MODE=no_ocr` no longer
bypasses these checks: an incomplete native text layer is rejected so the
prior cache is retained. GPU modes (`balanced`, `vllm`) need additional Surya
inference-server prerequisites that this CPU image does not provide.

Suggested minimum for small feeds: 4 CPU cores, 8 GiB RAM and 10 GiB free
disk for the image and model cache. Marker conversion is CPU-heavy; keep the
worker concurrency at 1 unless the host has been sized for more.

## Local tests

The contract and HTTP tests use a fake converter and do not import Marker:

```sh
python3 -m unittest discover -s marker_worker/tests -v
```

Verify the Compose supervisor restarts the worker after a timeout, including a
client disconnect, and that the restarted worker converts the next request:

```sh
./scripts/marker-supervisor-test.sh
```

Build and run the real worker with the Compose profile:

```sh
docker compose --profile marker build marker
docker compose --profile marker up -d marker
docker compose ps marker
```

Then set `MARKER_WORKER_URL=http://marker:8000` for the API. A Pdfium-only
deployment does not start this service and leaves `PDF_EXTRACTION_ENGINE` at
`pdfium`.

## Rollback

Select `pdfium` globally (`PDF_EXTRACTION_ENGINE=pdfium`) or per feed, then
stop the worker with `docker compose --profile marker stop marker`. The next
refresh reprocesses affected documents with Pdfium because the engine is part
of the cache identity. Existing cached content remains available if a refresh
fails. The `marker_models` volume can be removed after the worker is no longer
needed.
