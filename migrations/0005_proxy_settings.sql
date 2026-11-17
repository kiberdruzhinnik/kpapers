ALTER TABLE feeds
    ADD COLUMN proxy_mode TEXT NOT NULL DEFAULT 'inherit'
        CHECK (proxy_mode IN ('inherit', 'direct', 'custom'));

ALTER TABLE feeds
    ADD COLUMN proxy_url TEXT;

CREATE TABLE application_settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
