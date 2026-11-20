#![no_main]
#![allow(dead_code)]

#[path = "../../src/auth.rs"]
mod auth;
#[path = "../../src/feed_api.rs"]
mod feed_api;
#[path = "../../src/pdf_processing.rs"]
mod pdf_processing;
#[path = "../../src/providers/mod.rs"]
mod providers;
#[path = "../../src/proxy.rs"]
mod proxy;
#[path = "../../src/rss.rs"]
mod rss;

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let Some((&selector, input)) = data.split_first() else {
        return;
    };
    let text = String::from_utf8_lossy(input);

    match selector % 15 {
        0 => {
            let _ = serde_json::from_slice::<auth::LoginRequest>(input);
        }
        1 => {
            let _ = serde_json::from_slice::<feed_api::FeedInput>(input);
        }
        2 => {
            let _ = serde_json::from_slice::<feed_api::YearInput>(input);
        }
        3 => {
            let _ = serde_json::from_slice::<feed_api::MonthInput>(input);
        }
        4 => {
            let _ = serde_json::from_slice::<feed_api::PromotionOverride>(input);
        }
        5 => {
            let _ = serde_urlencoded::from_bytes::<feed_api::ForceQuery>(input);
        }
        6 => {
            let _ = providers::validate_source_url(&text);
            if let Ok(url) = url::Url::parse(&text) {
                let _ = providers::validate_download_url_shape(&url);
            }
        }
        7 => {
            let _ = serde_json::from_slice::<providers::yandex_disk::ResourcePage>(input);
        }
        8 => {
            let _ = serde_json::from_slice::<providers::yandex_disk::DownloadResponse>(input);
        }
        9 => {
            let _ = serde_json::from_slice::<
                providers::mail_cloud::ApiResponse<providers::mail_cloud::Folder>,
            >(input);
        }
        10 => {
            let _ = serde_json::from_slice::<
                providers::mail_cloud::ApiResponse<providers::mail_cloud::Dispatcher>,
            >(input);
        }
        11 => {
            let normalized = pdf_processing::normalize_text(&text);
            let _ = pdf_processing::classify_promotion(&text, &normalized);
            let mut extracted = pdf_processing::ExtractedText {
                full_text: normalized.clone(),
                html: String::new(),
                markdown: String::new(),
                first_page: normalized,
                page_count: 1,
            };
            pdf_processing::repair_sponsr_title_from_filename(&text, &mut extracted);
        }
        12 => {
            let _ = rss::parse_publication_date(&text);
        }
        13 => {
            let _ = rss::parse_date(&text);
        }
        _ => {
            let _ = auth::valid_session_cookie(&text, &[b'f'; 32], 1_800_000_000);
        }
    }
});
