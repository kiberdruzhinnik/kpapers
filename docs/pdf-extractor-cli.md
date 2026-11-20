# Export PDFs from public cloud folders

`kpapers-extract` downloads direct child PDFs from a supported public folder, applies the application's existing PDF cleanup, and writes one Markdown file per successfully processed PDF. It supports public links from Mail Cloud and Yandex Disk. Subfolders and non-PDF files are ignored, and the downloaded PDF bytes are not kept in the output directory.

## Run from a Rust checkout

From the repository root, pass the public folder URL followed by the output directory:

```sh
cargo run --release --bin kpapers-extract -- 'https://disk.yandex.ru/d/<public-token>' ./extracted
```

Mail Cloud links use the form `https://cloud.mail.ru/public/<owner>/<token>`. The output directory and any missing parent directories are created automatically. Existing Markdown files with the same generated name are replaced on a later run.

Each output file uses the upstream PDF's filename without its `.pdf` extension. Path separators and filesystem-reserved characters are replaced; if multiple files map to the same name, a stable suffix keeps their Markdown names distinct.

## Configuration and failures

- `KPAPERS_PROXY_URL` optionally routes provider requests through an `http://` or `socks5://` proxy with an explicit port. Malformed proxy configuration stops the command before provider requests.
- `MAX_PDF_BYTES` sets the maximum PDF size. It defaults to 25 MiB, matching application ingestion.
- PDFs without usable native text, failed downloads, and file write errors are reported individually. The command continues with other PDFs and exits unsuccessfully if any file fails. A folder listing or output-directory creation error stops the run.
- Scanned-only PDFs are not OCR processed; they are reported as failures.

For command usage, run `kpapers-extract --help`.

## Run the published Docker image

The release workflow publishes `kpapers-extractor` to GitHub Container Registry for `linux/amd64` and `linux/arm64`. Replace `<OWNER>` with the repository owner and mount a writable host directory at `/output`:

```sh
mkdir -p ./extracted
docker run --rm \
  --user "$(id -u):$(id -g)" \
  -v "$PWD/extracted:/output" \
  ghcr.io/<OWNER>/kpapers-extractor:latest \
  'https://disk.yandex.ru/d/<public-token>' /output
```

The container runs as UID:GID `568:568` by default. The example overrides that with the invoking host user so the generated files are writable without changing host-directory ownership. Alternatively, grant write permission on the mounted directory to UID:GID `568:568`. When a provider requires a proxy, pass it with `-e KPAPERS_PROXY_URL=...`; `MAX_PDF_BYTES` can also be set with `-e MAX_PDF_BYTES=...`.
