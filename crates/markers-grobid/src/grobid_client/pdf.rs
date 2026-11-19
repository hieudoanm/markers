//! PDF → TEI conversion via `processFulltextDocument`.

use std::path::Path;
use std::time::{Duration, Instant};

use crate::config;

/// Convert a PDF to TEI XML using the GROBID service.
pub async fn pdf_to_tei(
    client: &reqwest::Client,
    pdf_path: &Path,
    tei_path_fn: &(dyn Fn(&Path) -> std::path::PathBuf + Send + Sync),
) -> (u16, Option<std::path::PathBuf>, f64) {
    let started = Instant::now();
    let url = format!("{}/api/processFulltextDocument", config::grobid_url());

    let pdf_bytes = match std::fs::read(pdf_path) {
        Ok(b) => b,
        Err(e) => {
            tracing::error!("Failed to read PDF {}: {e}", pdf_path.display());
            return (0, None, elapsed(started));
        }
    };

    // One retry for transient failures (connection resets, GROBID 5xx);
    // 4xx responses are deterministic and not retried.
    for attempt in 0..2u32 {
        if attempt > 0 {
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        let part = reqwest::multipart::Part::bytes(pdf_bytes.clone())
            .file_name(
                pdf_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            )
            .mime_str("application/pdf")
            .unwrap_or_else(|_| reqwest::multipart::Part::text(""));

        let form = reqwest::multipart::Form::new()
            .text(
                "consolidateHeader",
                config::GROBID_CONSOLIDATE_HEADER.to_string(),
            )
            .part("input", part);

        let resp = match client.post(&url).multipart(form).send().await {
            Ok(resp) => resp,
            Err(e) => {
                tracing::error!("GROBID request failed (attempt {}): {e}", attempt + 1);
                if attempt == 1 {
                    return (0, None, elapsed(started));
                }
                continue;
            }
        };

        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        if status == 200 && !body.is_empty() {
            let out_path = tei_path_fn(pdf_path);
            if let Err(e) = std::fs::write(&out_path, body.as_bytes()) {
                tracing::error!("Failed to write TEI {}: {e}", out_path.display());
                return (status, None, elapsed(started));
            }
            return (status, Some(out_path), elapsed(started));
        }

        let snippet: String = body.chars().take(300).collect();
        tracing::error!(
            "GROBID returned {status} (attempt {}): {}",
            attempt + 1,
            if snippet.is_empty() {
                "<empty response>".to_string()
            } else {
                snippet
            }
        );
        // Retry server errors and empty responses; 4xx is deterministic.
        if (status < 500 && !body.is_empty()) || attempt == 1 {
            return (status, None, elapsed(started));
        }
    }
    (0, None, elapsed(started))
}

fn elapsed(start: Instant) -> f64 {
    start.elapsed().as_secs_f64()
}
