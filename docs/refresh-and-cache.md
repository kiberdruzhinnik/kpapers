# Refresh and cache operations

The single API instance checks for due linked months every five seconds. By default it refreshes the current calendar month every hour and older months every 24 hours. Set `CURRENT_MONTH_REFRESH_SECONDS` and `OLDER_MONTH_REFRESH_SECONDS` to change those defaults. A feed-level interval override replaces both defaults for that feed; the operator sets it in hours on the feed detail screen. The current month is determined from the API server's UTC calendar.

Each due or forced run is persisted in `refresh_jobs`. Month rows expose last attempt, next refresh, and the latest result in the operator view. **Fetch and parse** performs an immediate month scan; **Force refetch** queues a job that bypasses the unchanged-fingerprint cache check. Feed and year force buttons enqueue only their configured months.

## Engine-aware cache identity

Cached extracted text is keyed by the source fingerprint, the effective extraction engine (`pdfium` or `marker`) and the processing revision. The effective engine is resolved at each manual or scheduled extraction before the cache is checked. Consequently:

- an unchanged PDF with an unchanged engine reuses its cache;
- changing a feed override or the global `PDF_EXTRACTION_ENGINE` causes the next refresh to extract with the newly selected engine;
- a processing-revision change invalidates all cached text after a deployment;
- **Force refetch** bypasses the unchanged-source check but still uses the currently selected engine;
- a failed extraction records the failure and leaves the previous successful cached text and HTML in place. The API does not fall back to the other engine.

Engine changes are applied lazily on the next refresh; the deployment does not enqueue an immediate bulk reprocess.

## Backup and restore

The database and cached extracted text/HTML live together under `/data` in the named Compose volume. Back up the volume as a consistent SQLite snapshot: stop the Compose services, copy `/data/kpapers.sqlite` and its `-wal`/`-shm` companions if present, then restart the services. To restore, stop the services, replace the database files in the volume with the backup, and start the services so migrations run. Keep the backup directory restricted because it contains the operator's feed data and cached source text. Marker model files in `marker_models` are a re-downloadable cache and do not need to be included.
