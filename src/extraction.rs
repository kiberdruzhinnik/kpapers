use std::{sync::Arc, time::Duration};
use tokio::sync::Semaphore;

/// The PDF extraction backend used for a feed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtractionEngine {
    Pdfium,
    Marker,
}

impl ExtractionEngine {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pdfium => "pdfium",
            Self::Marker => "marker",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "pdfium" => Some(Self::Pdfium),
            "marker" => Some(Self::Marker),
            _ => None,
        }
    }

    /// Resolve a feed override, falling back to the deployment default.
    pub fn resolve(self, feed_override: Option<&str>) -> Self {
        feed_override.and_then(Self::parse).unwrap_or(self)
    }
}

/// Configuration for the optional Marker worker.
#[derive(Clone)]
pub struct MarkerSettings {
    pub endpoint: Option<String>,
    pub timeout: Duration,
    pub max_response_bytes: usize,
    /// Bounds concurrent requests made by this API process. The worker also
    /// bounds its own conversions, but this prevents a burst of manual
    /// extractions from queuing unbounded work.
    pub permits: Arc<Semaphore>,
}

impl Default for MarkerSettings {
    fn default() -> Self {
        Self {
            endpoint: None,
            timeout: Duration::from_secs(120),
            max_response_bytes: 8 * 1024 * 1024,
            permits: Arc::new(Semaphore::new(1)),
        }
    }
}

#[derive(Clone)]
pub struct ExtractionSettings {
    pub default_engine: ExtractionEngine,
    pub marker: MarkerSettings,
}

impl Default for ExtractionSettings {
    fn default() -> Self {
        Self {
            default_engine: ExtractionEngine::Pdfium,
            marker: MarkerSettings::default(),
        }
    }
}

impl ExtractionSettings {
    pub fn from_env() -> Result<Self, String> {
        let default_engine = match std::env::var("PDF_EXTRACTION_ENGINE") {
            Ok(value) if !value.trim().is_empty() => ExtractionEngine::parse(value.trim())
                .ok_or_else(|| {
                    format!("PDF_EXTRACTION_ENGINE must be pdfium or marker, got {value:?}")
                })?,
            _ => ExtractionEngine::Pdfium,
        };
        let endpoint = std::env::var("MARKER_WORKER_URL")
            .ok()
            .map(|value| value.trim().trim_end_matches('/').to_owned())
            .filter(|value| !value.is_empty());
        let timeout = Duration::from_secs(positive_number(
            std::env::var("MARKER_WORKER_TIMEOUT_SECONDS")
                .ok()
                .as_deref(),
            120,
        ) as u64);
        let max_response_bytes = positive_number(
            std::env::var("MARKER_WORKER_MAX_RESPONSE_BYTES")
                .ok()
                .as_deref(),
            8 * 1024 * 1024,
        );
        let concurrency = positive_number(
            std::env::var("MARKER_WORKER_MAX_CONCURRENCY")
                .ok()
                .as_deref(),
            1,
        );
        Ok(Self {
            default_engine,
            marker: MarkerSettings {
                endpoint,
                timeout,
                max_response_bytes,
                permits: Arc::new(Semaphore::new(concurrency)),
            },
        })
    }

    pub fn effective_engine(&self, feed_override: Option<&str>) -> ExtractionEngine {
        self.default_engine.resolve(feed_override)
    }
}

fn positive_number(value: Option<&str>, default: usize) -> usize {
    value
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::{ExtractionEngine, ExtractionSettings, positive_number};

    #[test]
    fn parses_only_supported_engine_names() {
        assert_eq!(
            ExtractionEngine::parse("pdfium"),
            Some(ExtractionEngine::Pdfium)
        );
        assert_eq!(
            ExtractionEngine::parse("marker"),
            Some(ExtractionEngine::Marker)
        );
        for invalid in ["", "Pdfium", "marker-pdf", "pdf-extract", "none"] {
            assert_eq!(ExtractionEngine::parse(invalid), None, "{invalid:?}");
        }
    }

    #[test]
    fn feed_override_wins_and_unknown_values_fall_back_to_the_default() {
        let settings = ExtractionSettings::default();
        assert_eq!(settings.effective_engine(None), ExtractionEngine::Pdfium);
        assert_eq!(
            settings.effective_engine(Some("marker")),
            ExtractionEngine::Marker
        );
        assert_eq!(
            settings.effective_engine(Some("pdfium")),
            ExtractionEngine::Pdfium
        );
        assert_eq!(
            settings.effective_engine(Some("unsupported")),
            ExtractionEngine::Pdfium
        );
    }

    #[test]
    fn bounded_settings_accept_only_positive_numbers() {
        assert_eq!(positive_number(Some("1"), 7), 1);
        assert_eq!(positive_number(Some("4096"), 7), 4096);
        for invalid in [
            None,
            Some(""),
            Some("0"),
            Some("-1"),
            Some("1.5"),
            Some("x"),
        ] {
            assert_eq!(positive_number(invalid, 7), 7, "{invalid:?}");
        }
    }
}
