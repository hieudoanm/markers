//! Local JVM backend: spawn `grobid-service` from the standalone GROBID
//! distribution (no Docker) and manage its lifecycle.

use std::path::Path;

use super::wait_alive;
use crate::utils::log;

/// Locally spawned GROBID service child, killed on CLI exit unless
/// `GROBID_LOCAL_KEEP=1` asked for a persistent service.
static LOCAL_SERVICE: std::sync::Mutex<Option<std::process::Child>> = std::sync::Mutex::new(None);

/// Spawn the standalone GROBID JVM service (`grobid-service/bin/grobid-service`)
/// as a child process and wait for the health endpoint. By default the
/// service is killed when the CLI exits; set `GROBID_LOCAL_KEEP=1` to leave
/// it running (e.g. for repeated conversions). stdout/stderr go to
/// `grobid-service/grobid-service.log` inside the install.
pub(super) async fn start_local_service(bin: &Path, probe: &reqwest::Client) -> bool {
    // Note: if an instance is already booting, a duplicate spawn simply fails
    // to bind the port and exits — wait_alive() still succeeds once the
    // original comes up.
    // Service log lives inside the grobid-service dir; fresh file per spawn.
    let log_path = bin
        .parent()
        .and_then(|p| p.parent())
        .map(|dir| dir.join("grobid-service.log"));
    if let Some(path) = &log_path {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::remove_file(path);
    }
    let open_log = || -> Option<std::process::Stdio> {
        let path = log_path.as_ref()?;
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .ok()
            .map(std::process::Stdio::from)
    };
    let stdout = open_log();
    let stderr = open_log();

    let mut cmd = std::process::Command::new(bin);
    // The service resolves `grobid-home/config/grobid.yaml` relative to the
    // working directory; run it from the install root (parent of
    // `grobid-service/`) for the standard distribution layout.
    if let Some(root) = bin
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
    {
        cmd.current_dir(root);
    }
    cmd.stdin(std::process::Stdio::null())
        .stdout(stdout.unwrap_or_else(std::process::Stdio::null))
        .stderr(stderr.unwrap_or_else(std::process::Stdio::null));

    match cmd.spawn() {
        Ok(child) => {
            log().info(&format!(
                "Starting GROBID local service {}...",
                bin.display()
            ));
            let keep = std::env::var("GROBID_LOCAL_KEEP").is_ok_and(|v| v == "1" || v == "true");
            if !keep {
                if let Ok(mut guard) = LOCAL_SERVICE.lock() {
                    *guard = Some(child);
                }
            }
            // keep=true: dropping the Child detaches the service.
            wait_alive(
                probe,
                180,
                "local service (first start loads CRF models — can take a few minutes)",
            )
            .await
        }
        Err(e) => {
            log().error(&format!(
                "Failed to spawn GROBID local service {}: {e}",
                bin.display()
            ));
            false
        }
    }
}

/// Kill the local GROBID service if this CLI spawned it. No-op for
/// externally managed instances (Docker container, remote URL, or a
/// service the user started themselves).
pub fn stop_local_service() {
    if let Ok(mut guard) = LOCAL_SERVICE.lock() {
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
