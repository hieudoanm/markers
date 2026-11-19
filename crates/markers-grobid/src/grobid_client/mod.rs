//! GROBID service backends: health checks, backend selection
//! (local JVM service → Docker → external URL), and startup orchestration.

mod docker;
mod local;
mod pdf;

use std::time::Duration;

use crate::config;
use crate::utils::log;

pub use local::stop_local_service;
pub use pdf::pdf_to_tei;

/// How long to wait for a single Docker CLI invocation before giving up.
const DOCKER_CMD_TIMEOUT: Duration = Duration::from_secs(30);

/// How long to wait for `docker info` when probing the daemon.
const DOCKER_INFO_TIMEOUT: Duration = Duration::from_secs(15);

/// Short-timeout client for health probes; safe to reuse across poll loops.
fn health_probe_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

pub(crate) async fn alive(client: &reqwest::Client) -> bool {
    match client.get(config::health_url()).send().await {
        Ok(resp) => resp.status().as_u16() == 200,
        Err(_) => false,
    }
}

/// Check if the GROBID server is alive.
pub async fn is_grobid_alive() -> bool {
    alive(&health_probe_client()).await
}

/// Ensure a GROBID backend is reachable, starting one if needed.
///
/// Precedence: an already-running instance is reused; `GROBID_URL` means an
/// external instance that is never managed; a locally installed JVM service
/// (`GROBID_LOCAL_BIN` or the default path) is spawned on demand; otherwise
/// the Docker container is started.
pub async fn start_grobid() -> bool {
    let probe = health_probe_client();
    if alive(&probe).await {
        log().info(&format!(
            "GROBID already running at {}",
            config::grobid_url()
        ));
        return true;
    }

    // A custom GROBID_URL implies a remote/external instance — never try to
    // manage a local process for it.
    if std::env::var("GROBID_URL").is_ok() {
        log().warn(&format!(
            "GROBID_URL is set but {} is not reachable",
            config::grobid_url()
        ));
        return false;
    }

    // Docker-free mode: a locally installed GROBID JVM service
    // (GROBID_LOCAL_BIN or the default ~/.local/share/grobid.rs path).
    if let Some(bin) = config::grobid_local_bin() {
        return local::start_local_service(&bin, &probe).await;
    }

    docker::start_docker_container(&probe).await
}

/// Poll the health endpoint for up to `secs` seconds, logging progress
/// every 15s so long JVM warmups don't look like a hang. `what` describes
/// the backend being waited on (container name, local service, ...).
pub(crate) async fn wait_alive(probe: &reqwest::Client, secs: u64, what: &str) -> bool {
    for i in 0..secs {
        if alive(probe).await {
            log().info(&format!("GROBID ready after {}s", i + 1));
            return true;
        }
        if i > 0 && i % 15 == 0 {
            log().info(&format!(
                "{what}: still waiting for GROBID... ({}s elapsed)",
                i
            ));
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    log().error(&format!("{what}: GROBID did not become ready in {secs}s"));
    false
}
