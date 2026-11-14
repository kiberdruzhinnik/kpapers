-- A NULL value means the feed inherits the deployment-wide default engine.
ALTER TABLE feeds
    ADD COLUMN extraction_engine TEXT
        CHECK (extraction_engine IS NULL OR extraction_engine IN ('pdfium', 'marker'));
