use kpapers_api::{pdf_processing, providers, proxy};
use sha2::{Digest, Sha256};
use std::{
    env,
    path::{Path, PathBuf},
    process::ExitCode,
};

const DEFAULT_MAX_PDF_BYTES: usize = 25 * 1024 * 1024;
const USAGE: &str = "Usage: kpapers-extract <PUBLIC_FOLDER_URL> <OUTPUT_DIR>\n\
Supported URLs: https://cloud.mail.ru/public/<owner>/<token> and https://disk.yandex.ru/d/<token>";

#[derive(Debug, PartialEq, Eq)]
struct CliArgs {
    public_url: String,
    output_dir: PathBuf,
}

#[derive(Debug)]
struct FileResult {
    source_name: String,
    output_name: Option<String>,
    error: Option<String>,
}

fn parse_args(args: &[String]) -> Result<Option<CliArgs>, String> {
    if args == ["--help"] || args == ["-h"] {
        return Ok(None);
    }
    if args.len() != 2 {
        return Err(USAGE.to_owned());
    }
    Ok(Some(CliArgs {
        public_url: args[0].clone(),
        output_dir: PathBuf::from(&args[1]),
    }))
}

#[tokio::main]
async fn main() -> ExitCode {
    match parse_args(&env::args().skip(1).collect::<Vec<_>>()) {
        Ok(None) => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("kpapers-extract: {error}");
            ExitCode::FAILURE
        }
        Ok(Some(args)) => match run(args).await {
            Ok(failed) => exit_code(failed),
            Err(error) => {
                eprintln!("kpapers-extract: {}", safe_error_text(&error));
                ExitCode::FAILURE
            }
        },
    }
}

async fn run(args: CliArgs) -> Result<bool, String> {
    let provider = providers::validate_source_url(&args.public_url).map_err(str::to_owned)?;
    let use_fixtures = args.public_url.starts_with("fixture:");
    proxy::validate_environment().map_err(|error| error.to_string())?;
    let proxy = configured_proxy()?;
    let maximum_bytes = maximum_pdf_bytes();
    ensure_output_directory(&args.output_dir).await?;

    let files = if use_fixtures {
        providers::local_fixtures::list_root_pdfs(&args.public_url)
            .await
            .map_err(|error| format!("source listing failed: {}", safe_error_text(&error)))?
    } else {
        match provider {
            "mail_cloud" => providers::mail_cloud::list_root_pdfs(&args.public_url, &proxy)
                .await
                .map_err(|error| {
                    format!(
                        "source listing failed: {}",
                        safe_error_text(&proxy.redact_text(&error.to_string()))
                    )
                })?,
            "yandex_disk" => providers::yandex_disk::list_root_pdfs(&args.public_url, &proxy)
                .await
                .map_err(|error| {
                    format!(
                        "source listing failed: {}",
                        safe_error_text(&proxy.redact_text(&error.to_string()))
                    )
                })?,
            _ => return Err("unsupported public folder provider".into()),
        }
    };

    if files.is_empty() {
        println!("No direct PDF files found.");
        println!("Processed 0 PDF files; 0 failed.");
        return Ok(false);
    }

    let file_results = process_files(&files, &args.output_dir, maximum_bytes as u64, {
        let proxy = proxy.clone();
        move |file| {
            let proxy = proxy.clone();
            async move {
                if use_fixtures {
                    return providers::local_fixtures::download(&file, maximum_bytes as u64).await;
                }
                let result = match provider {
                    "mail_cloud" => {
                        providers::mail_cloud::download(&file, maximum_bytes as u64, &proxy).await
                    }
                    "yandex_disk" => {
                        providers::yandex_disk::download(&file, maximum_bytes as u64, &proxy).await
                    }
                    _ => unreachable!("provider was validated above"),
                };
                result.map_err(|error| safe_error_text(&proxy.redact_text(&error.to_string())))
            }
        }
    })
    .await;

    let mut successes = 0;
    let mut failures = 0;
    for result in &file_results {
        if let Some(message) = file_failure_message(result) {
            failures += 1;
            eprintln!("{message}");
        } else {
            successes += 1;
            println!(
                "Wrote {} -> {}",
                result.source_name,
                result
                    .output_name
                    .as_deref()
                    .expect("successful file has an output name")
            );
        }
    }
    println!(
        "{}",
        completion_summary(successes + failures, successes, failures)
    );
    Ok(failures > 0)
}

fn configured_proxy() -> Result<proxy::EffectiveProxy, String> {
    match env::var("KPAPERS_PROXY_URL") {
        Ok(value) if !value.trim().is_empty() => proxy::ProxyUrl::parse(&value)
            .map(proxy::EffectiveProxy::Proxy)
            .map_err(|error| error.to_string()),
        Ok(_) | Err(env::VarError::NotPresent) => Ok(proxy::EffectiveProxy::Direct),
        Err(env::VarError::NotUnicode(_)) => Err("proxy URL must be valid UTF-8".into()),
    }
}

fn maximum_pdf_bytes() -> usize {
    env::var("MAX_PDF_BYTES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_MAX_PDF_BYTES)
}

fn exit_code(failed: bool) -> ExitCode {
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

async fn ensure_output_directory(output_dir: &Path) -> Result<(), String> {
    tokio::fs::create_dir_all(output_dir)
        .await
        .map_err(|error| format!("could not create output directory: {error}"))
}

fn completion_summary(total: usize, successes: usize, failures: usize) -> String {
    format!("Processed {total} PDF files; {successes} succeeded, {failures} failed.")
}

fn file_failure_message(result: &FileResult) -> Option<String> {
    result
        .error
        .as_ref()
        .map(|error| format!("Failed {}: {error}", result.source_name))
}

fn safe_display_name(filename: &str) -> String {
    filename
        .chars()
        .map(|ch| if ch.is_control() { '\u{fffd}' } else { ch })
        .collect()
}

async fn process_files<F, Fut>(
    files: &[providers::RemotePdf],
    output_dir: &Path,
    maximum_bytes: u64,
    download: F,
) -> Vec<FileResult>
where
    F: Fn(providers::RemotePdf) -> Fut,
    Fut: std::future::Future<Output = Result<Vec<u8>, String>>,
{
    let output_names = output_names(files);
    let mut results = Vec::with_capacity(files.len());
    for (file, output_name) in files.iter().cloned().zip(output_names) {
        let result = async {
            let bytes = download(file.clone()).await?;
            let mut extracted = pdf_processing::extract_pdf(&bytes, maximum_bytes as usize)?;
            pdf_processing::repair_sponsr_title_from_filename(&file.filename, &mut extracted);
            tokio::fs::write(output_dir.join(&output_name), extracted.markdown)
                .await
                .map_err(|error| format!("could not write Markdown output: {error}"))?;
            Ok::<(), String>(())
        }
        .await;
        results.push(match result {
            Ok(()) => FileResult {
                source_name: safe_display_name(&file.filename),
                output_name: Some(output_name),
                error: None,
            },
            Err(error) => FileResult {
                source_name: safe_display_name(&file.filename),
                output_name: None,
                error: Some(safe_error_text(&error)),
            },
        });
    }
    results
}

fn output_names(files: &[providers::RemotePdf]) -> Vec<String> {
    let stems = files
        .iter()
        .map(|file| safe_stem(&file.filename))
        .collect::<Vec<_>>();
    let mut counts = std::collections::HashMap::<String, usize>::new();
    for stem in &stems {
        *counts.entry(stem.to_lowercase()).or_default() += 1;
    }

    let mut used = std::collections::HashSet::new();
    files
        .iter()
        .zip(stems)
        .enumerate()
        .map(|(index, (file, stem))| {
            let mut name = if counts[&stem.to_lowercase()] > 1 {
                let digest = Sha256::digest(format!("{}\0{}", file.provider_id, file.path));
                format!(
                    "{stem}-{:02x}{:02x}{:02x}{:02x}.md",
                    digest[0], digest[1], digest[2], digest[3]
                )
            } else {
                format!("{stem}.md")
            };
            if !used.insert(name.to_lowercase()) {
                let digest = Sha256::digest(format!("{}\0{}", file.provider_id, file.path));
                let suffix = format!(
                    "{:02x}{:02x}{:02x}{:02x}",
                    digest[0], digest[1], digest[2], digest[3]
                );
                let mut disambiguator = index;
                loop {
                    name = format!("{stem}-{suffix}-{disambiguator}.md");
                    if used.insert(name.to_lowercase()) {
                        break;
                    }
                    disambiguator += 1;
                }
            }
            name
        })
        .collect()
}

fn safe_stem(filename: &str) -> String {
    let stem = filename
        .rsplit_once('.')
        .filter(|(_, extension)| extension.eq_ignore_ascii_case("pdf"))
        .map(|(stem, _)| stem)
        .unwrap_or(filename);
    let sanitized = stem
        .chars()
        .map(|ch| {
            if ch.is_control() || matches!(ch, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
            {
                '_'
            } else {
                ch
            }
        })
        .collect::<String>();
    let sanitized = sanitized.trim().trim_matches('.').trim().to_owned();
    if sanitized.is_empty() || sanitized == "." || sanitized == ".." {
        "document".into()
    } else {
        sanitized
    }
}

fn safe_error_text(error: &str) -> String {
    error
        .split_whitespace()
        .map(|part| {
            if part.contains("://") {
                "[URL redacted]"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(500)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn remote_pdf(filename: &str, provider_id: &str) -> providers::RemotePdf {
        providers::RemotePdf {
            provider_id: provider_id.into(),
            path: filename.into(),
            filename: filename.into(),
            size: 0,
            modified: None,
            fingerprint: String::new(),
            download_url: String::new(),
            public_url: String::new(),
        }
    }

    fn temporary_directory() -> PathBuf {
        env::temp_dir().join(format!("kpapers-extract-test-{}", uuid::Uuid::new_v4()))
    }

    #[test]
    fn parses_two_positional_arguments_and_help() {
        let args = vec!["https://disk.yandex.ru/d/token".into(), "out".into()];
        assert_eq!(
            parse_args(&args).unwrap(),
            Some(CliArgs {
                public_url: args[0].clone(),
                output_dir: PathBuf::from("out"),
            })
        );
        assert!(parse_args(&[]).is_err());
        assert!(parse_args(&["url".into()]).is_err());
        assert!(parse_args(&["url".into(), "out".into(), "extra".into()]).is_err());
        assert_eq!(parse_args(&["--help".into()]).unwrap(), None);
    }

    #[test]
    fn sanitizes_output_names_and_resolves_collisions_deterministically() {
        let files = vec![
            remote_pdf("../study.pdf", "provider-a"),
            remote_pdf("..\\study.pdf", "provider-b"),
            remote_pdf("...pdf", "provider-c"),
        ];
        let first = output_names(&files);
        assert_eq!(first, output_names(&files));
        assert_eq!(first.len(), 3);
        assert_ne!(first[0], first[1]);
        assert!(
            first.iter().all(|name| {
                !name.contains('/') && !name.contains('\\') && !name.starts_with('.')
            })
        );
        assert!(
            first
                .iter()
                .all(|name| Path::new(name).components().count() == 1)
        );
    }

    #[tokio::test]
    async fn creates_missing_output_directories_and_continues_after_file_failures() {
        let output_dir = temporary_directory().join("nested/output");
        ensure_output_directory(&output_dir).await.unwrap();
        assert!(output_dir.is_dir());
        let files = vec![
            remote_pdf("article-alpha.pdf", "good"),
            remote_pdf("broken.pdf", "bad"),
        ];
        let good_pdf = include_bytes!("../../fuzz/corpus/pdf_bytes/article-alpha.pdf").to_vec();
        let results = process_files(
            &files,
            &output_dir,
            DEFAULT_MAX_PDF_BYTES as u64,
            move |file| {
                let good_pdf = good_pdf.clone();
                async move {
                    if file.provider_id == "good" {
                        Ok(good_pdf)
                    } else {
                        Err("download failed".into())
                    }
                }
            },
        )
        .await;
        assert!(results[0].error.is_none());
        assert_eq!(results[1].error.as_deref(), Some("download failed"));
        assert_eq!(
            file_failure_message(&results[1]).as_deref(),
            Some("Failed broken.pdf: download failed")
        );
        assert_eq!(
            completion_summary(2, 1, 1),
            "Processed 2 PDF files; 1 succeeded, 1 failed."
        );
        assert_eq!(
            exit_code(results.iter().any(|result| result.error.is_some())),
            ExitCode::FAILURE
        );
        assert_eq!(
            exit_code(results.iter().all(|result| result.error.is_none())),
            ExitCode::SUCCESS
        );
        let output = tokio::fs::read_to_string(output_dir.join("article-alpha.md"))
            .await
            .unwrap();
        assert!(!output.trim().is_empty());
        let _ = tokio::fs::remove_dir_all(output_dir.parent().unwrap().parent().unwrap()).await;
    }

    #[tokio::test]
    async fn reports_an_unwritable_output_path() {
        let parent = temporary_directory();
        tokio::fs::create_dir_all(&parent).await.unwrap();
        let file_path = parent.join("not-a-directory");
        tokio::fs::write(&file_path, "file").await.unwrap();
        let error = ensure_output_directory(&file_path.join("child"))
            .await
            .unwrap_err();
        assert!(error.starts_with("could not create output directory:"));
        let _ = tokio::fs::remove_dir_all(parent).await;
    }

    #[test]
    fn redacts_urls_from_diagnostics() {
        assert_eq!(
            safe_error_text("request failed at https://host.example/path?secret=1 timeout"),
            "request failed at [URL redacted] timeout"
        );
    }
}
