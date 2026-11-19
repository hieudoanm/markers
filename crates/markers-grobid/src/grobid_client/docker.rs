//! Docker backend: container lifecycle for the `grobid/grobid` image.

use std::path::PathBuf;

use super::{DOCKER_CMD_TIMEOUT, DOCKER_INFO_TIMEOUT, alive, wait_alive};
use crate::config;
use crate::utils::log;

fn which(cmd: &str) -> Option<PathBuf> {
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            let candidate = dir.join(cmd);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Run a Docker CLI command with a timeout; returns the exit status on success.
async fn docker_status(args: &[String]) -> Option<std::process::ExitStatus> {
    let timeout = if args.first().map(String::as_str) == Some("info") {
        DOCKER_INFO_TIMEOUT
    } else {
        DOCKER_CMD_TIMEOUT
    };
    let mut cmd = tokio::process::Command::new("docker");
    cmd.args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true);
    match tokio::time::timeout(timeout, cmd.status()).await {
        Ok(Ok(status)) => Some(status),
        _ => None,
    }
}

/// Run a Docker CLI command capturing output; returns trimmed stdout on success
/// or the stderr/exit info as an error string for diagnostics.
async fn docker_capture(args: &[String]) -> Result<String, String> {
    let timeout = if args.first().map(String::as_str) == Some("info") {
        DOCKER_INFO_TIMEOUT
    } else {
        DOCKER_CMD_TIMEOUT
    };
    let mut cmd = tokio::process::Command::new("docker");
    cmd.args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    let output = match tokio::time::timeout(timeout, cmd.output()).await {
        Ok(Ok(o)) => o,
        Ok(Err(e)) => return Err(format!("failed to spawn docker: {e}")),
        Err(_) => return Err("docker command timed out".to_string()),
    };
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(if stderr.is_empty() {
        format!("docker exited with {:?}", output.status.code())
    } else {
        stderr
    })
}

/// Check whether the Docker daemon is reachable (not just the CLI installed).
async fn docker_daemon_ready() -> bool {
    docker_status(&["info".to_string()])
        .await
        .is_some_and(|s| s.success())
}

/// Start the GROBID Docker container if not already running.
pub(super) async fn start_docker_container(probe: &reqwest::Client) -> bool {
    if which("docker").is_none() {
        log().warn(&format!(
            "Docker not found. Start GROBID manually at {}",
            config::grobid_url()
        ));
        return alive(probe).await;
    }

    // The CLI being on PATH is not enough — bail out fast if the daemon is
    // down or wedged instead of blocking on `docker run`.
    if !docker_daemon_ready().await {
        log().error(&format!(
            "Docker daemon is not reachable. Start Docker or run GROBID manually at {}",
            config::grobid_url()
        ));
        return alive(probe).await;
    }

    let container = config::grobid_container();
    let image = config::grobid_image();
    let port = config::grobid_port();

    // A container that is already running may simply still be booting —
    // killing it would restart the whole JVM warmup. Wait for it first.
    let inspect_args = vec![
        "inspect".to_string(),
        "-f".to_string(),
        "{{.State.Running}}".to_string(),
        container.clone(),
    ];
    if let Ok(state) = docker_capture(&inspect_args).await {
        if state == "true" {
            log().info(&format!(
                "Container {container} is running; waiting for GROBID to come up..."
            ));
            if wait_alive(probe, 90, &format!("container {container}")).await {
                return true;
            }
            log().warn(&format!(
                "Container {container} did not become ready; recreating it"
            ));
        }
    }

    // Remove any existing (stopped or wedged) container.
    let _ = docker_status(&["rm".into(), "-f".into(), container.clone()]).await;

    let run_args = vec![
        "run".to_string(),
        "-d".to_string(),
        "--rm".to_string(),
        "--init".to_string(),
        "--ulimit".to_string(),
        "core=0".to_string(),
        "-e".to_string(),
        "JAVA_TOOL_OPTIONS=-Xms1g -Xmx4g".to_string(),
        "-p".to_string(),
        format!("{}:8070", port),
        "--name".to_string(),
        container.clone(),
        image,
    ];

    if let Err(e) = docker_capture(&run_args).await {
        log().error(&format!(
            "Failed to start GROBID container {container}: {e}"
        ));
        return false;
    }

    log().info(&format!("Starting GROBID container {container}..."));
    wait_alive(probe, 120, &format!("container {container}")).await
}
