# Refresh and cache operations

The single API instance checks for due linked months every five seconds and automatically refreshes only the configured month matching the API server's current UTC year and month. The current month refreshes every hour by default; set `CURRENT_MONTH_REFRESH_SECONDS` to change that interval. A feed-level interval override replaces the default for the current month of that feed; the operator sets it in hours on the feed detail screen. Older months are not refreshed automatically and have no next scheduled refresh, but remain available for manual **Fetch and parse** or **Force refetch**. When the API server's UTC calendar advances, the newly current configured month becomes eligible for automatic refresh.

Each due or forced run is persisted in `refresh_jobs`. Month rows expose last attempt, next refresh, and the latest result in the operator view. **Fetch and parse** performs an immediate month scan; **Force refetch** queues a job that bypasses the unchanged-fingerprint cache check. Feed and year force buttons enqueue only their configured months.

## Proxy routing for refresh traffic

Provider discovery and article PDF downloads use the proxy selected for the feed. The operator can set a global proxy at `/manage/settings` and override it per feed with **Inherit global settings**, **Connect directly**, or **Use a custom proxy**. Inherited feeds resolve a custom feed proxy first, then the global GUI setting, then `KPAPERS_PROXY_URL`, and finally connect directly. Direct mode bypasses both shared settings. Supported URLs are `http://host:port` and `socks5://host:port`, optionally with `user:password@` credentials. Proxy routing does not relax destination validation, redirect checks, timeouts, or response size limits.

## PDF extraction cache identity

Cached extracted text is keyed by the source fingerprint, the `pdf_oxide` engine identity, and the processing revision. Consequently:

- unchanged PDFs with an unchanged processing revision reuse their cache;
- content extracted by older engines is reprocessed with `pdf_oxide` on refresh;
- a processing-revision change invalidates cached text after deployment;
- **Force refetch** bypasses the unchanged-source check;
- a failed extraction leaves the last successful cached text and HTML in place.

Reprocessing is lazy and happens on the next scheduled or manual refresh.

## Backup and restore

The database and cached extracted text/HTML live together under `/data` in the named Compose volume. Back up the volume as a consistent SQLite snapshot: stop the Compose services, copy `/data/kpapers.sqlite` and its `-wal`/`-shm` companions if present, then restart the services. To restore, stop the services, replace the database files in the volume with the backup, and start the services so migrations run. Keep the backup directory restricted because it contains the operator's feed data and cached source text.
