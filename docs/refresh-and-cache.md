# Refresh and cache operations

The single API instance checks for due linked months every five seconds. By default it refreshes the current calendar month every hour and older months every 24 hours. Set `CURRENT_MONTH_REFRESH_SECONDS` and `OLDER_MONTH_REFRESH_SECONDS` to change those defaults. A feed-level interval override replaces both defaults for that feed; the operator sets it in hours on the feed detail screen. The current month is determined from the API server's UTC calendar.

Each due or forced run is persisted in `refresh_jobs`. Month rows expose last attempt, next refresh, and the latest result in the operator view. **Fetch and parse** performs an immediate month scan; **Force refetch** queues a job that bypasses the unchanged-fingerprint cache check. Feed and year force buttons enqueue only their configured months.

The database and cached extracted text/HTML live together under `/data` in the named Compose volume. Back up the volume as a consistent SQLite snapshot: stop the Compose services, copy `/data/kpapers.sqlite` and its `-wal`/`-shm` companions if present, then restart the services. To restore, stop the services, replace the database files in the volume with the backup, and start the services so migrations run. Keep the backup directory restricted because it contains the operator's feed data and cached source text.
