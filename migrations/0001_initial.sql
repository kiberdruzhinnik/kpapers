PRAGMA foreign_keys = ON;

CREATE TABLE feeds (
    id TEXT PRIMARY KEY NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    active INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    refresh_interval_seconds INTEGER CHECK (refresh_interval_seconds IS NULL OR refresh_interval_seconds > 0),
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE feed_years (
    feed_id TEXT NOT NULL,
    year INTEGER NOT NULL CHECK (year BETWEEN 1 AND 9999),
    PRIMARY KEY (feed_id, year),
    FOREIGN KEY (feed_id) REFERENCES feeds(id) ON DELETE CASCADE
);

CREATE TABLE feed_months (
    id TEXT PRIMARY KEY NOT NULL,
    feed_id TEXT NOT NULL,
    year INTEGER NOT NULL,
    month INTEGER NOT NULL CHECK (month BETWEEN 1 AND 12),
    provider TEXT NOT NULL CHECK (provider IN ('yandex_disk', 'mail_cloud')),
    source_url TEXT NOT NULL,
    last_attempt_at TEXT,
    next_refresh_at TEXT,
    last_result TEXT CHECK (last_result IS NULL OR last_result IN ('success', 'failure')),
    last_error TEXT,
    UNIQUE (feed_id, year, month),
    FOREIGN KEY (feed_id, year) REFERENCES feed_years(feed_id, year) ON DELETE CASCADE
);

CREATE TABLE source_documents (
    id TEXT PRIMARY KEY NOT NULL,
    month_id TEXT NOT NULL,
    provider_file_id TEXT,
    source_path TEXT NOT NULL,
    filename TEXT NOT NULL,
    source_url TEXT NOT NULL,
    content_fingerprint TEXT NOT NULL,
    provider_revision TEXT,
    size_bytes INTEGER NOT NULL CHECK (size_bytes >= 0),
    provider_modified_at TEXT,
    publication_date TEXT,
    promotional_classification TEXT NOT NULL DEFAULT 'publish'
        CHECK (promotional_classification IN ('publish', 'promotional')),
    promotional_override TEXT
        CHECK (promotional_override IS NULL OR promotional_override IN ('include', 'exclude')),
    extraction_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (extraction_status IN ('pending', 'success', 'failure')),
    last_error TEXT,
    discovered_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (month_id, source_path),
    FOREIGN KEY (month_id) REFERENCES feed_months(id) ON DELETE CASCADE
);

CREATE TABLE cached_content (
    document_id TEXT PRIMARY KEY NOT NULL,
    content_fingerprint TEXT NOT NULL,
    extracted_text TEXT NOT NULL,
    first_page_text TEXT NOT NULL,
    sanitized_html TEXT NOT NULL,
    page_count INTEGER NOT NULL CHECK (page_count >= 0),
    processed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (document_id) REFERENCES source_documents(id) ON DELETE CASCADE
);

CREATE TABLE refresh_jobs (
    id TEXT PRIMARY KEY NOT NULL,
    scope TEXT NOT NULL CHECK (scope IN ('feed', 'year', 'month')),
    feed_id TEXT NOT NULL,
    year INTEGER,
    month_id TEXT,
    force INTEGER NOT NULL DEFAULT 0 CHECK (force IN (0, 1)),
    status TEXT NOT NULL DEFAULT 'queued'
        CHECK (status IN ('queued', 'running', 'completed', 'failed')),
    requested_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    started_at TEXT,
    finished_at TEXT,
    error TEXT,
    FOREIGN KEY (feed_id) REFERENCES feeds(id) ON DELETE CASCADE,
    FOREIGN KEY (feed_id, year) REFERENCES feed_years(feed_id, year) ON DELETE CASCADE,
    FOREIGN KEY (month_id) REFERENCES feed_months(id) ON DELETE CASCADE
);

CREATE INDEX idx_feed_months_due ON feed_months(next_refresh_at);
CREATE INDEX idx_source_documents_month ON source_documents(month_id);
CREATE INDEX idx_refresh_jobs_status_requested ON refresh_jobs(status, requested_at);
