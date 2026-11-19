//! markers desktop backend: Tauri commands that drive `markers-grobid`
//! (GROBID service → TEI → Markdown) for the app's dropzone UI.

use std::path::{Path, PathBuf};
use std::time::Duration;

use markers_grobid::grobid_client;
use markers_grobid::metadata::extract_metadata_from_tei;
use markers_grobid::render::convert_tei_to_md;
use serde::{Deserialize, Serialize};
use tauri_plugin_dialog::DialogExt;

/// Per-file conversion outcome returned to the frontend.
#[derive(Debug, Serialize)]
pub struct FileOutcome {
    pub name: String,
    pub ok: bool,
    pub error: Option<String>,
    pub tei_path: Option<String>,
    pub md_path: Option<String>,
}

/// Outcome for browser-style uploads (bytes → temp files → converted
/// contents returned inline).
#[derive(Debug, Serialize)]
pub struct UploadOutcome {
    pub name: String,
    pub ok: bool,
    pub error: Option<String>,
    pub tei_xml: Option<String>,
    pub md: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UploadFile {
    pub name: String,
    pub data: Vec<u8>,
}

fn pdf_stem(path: &Path) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "untitled".to_string())
}

/// Default output dir: `markers-out/` next to the first PDF.
fn default_out_dir(paths: &[String]) -> Result<PathBuf, String> {
    let first = Path::new(&paths[0]);
    let parent = first
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    Ok(parent.join("markers-out"))
}

/// Convert one PDF: ensure TEI (cached if present), render Markdown,
/// write both into `out_dir`. Returns (tei_path, md_path).
async fn convert_one(
    client: &reqwest::Client,
    pdf: &Path,
    out_dir: &Path,
) -> Result<(PathBuf, PathBuf), String> {
    let stem = pdf_stem(pdf);
    let tei_path = out_dir.join(format!("{stem}.tei.xml"));

    if !tei_path.exists() {
        let (status, out, _) = grobid_client::pdf_to_tei(client, pdf, &|_| tei_path.clone()).await;
        if !(status == 200 && out.is_some()) {
            return Err(format!("GROBID returned status {status}"));
        }
    }

    let meta = extract_metadata_from_tei(&tei_path).unwrap_or_default();
    let md = convert_tei_to_md(&tei_path, Some(&meta)).map_err(|e| e.to_string())?;
    let md_path = out_dir.join(format!("{stem}.md"));
    std::fs::write(&md_path, md).map_err(|e| format!("write {}: {e}", md_path.display()))?;
    Ok((tei_path, md_path))
}

async fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| e.to_string())
}

async fn ensure_grobid() -> Result<(), String> {
    if !grobid_client::start_grobid().await && !grobid_client::is_grobid_alive().await {
        return Err(
            "GROBID is not available — install Docker or run scripts/setup-grobid-local.sh"
                .to_string(),
        );
    }
    Ok(())
}

/// Native file picker for PDFs (multi-select).
#[tauri::command]
fn pick_pdfs(app: tauri::AppHandle) -> Vec<String> {
    app.dialog()
        .file()
        .add_filter("PDF", &["pdf"])
        .blocking_pick_files()
        .map(|paths| {
            paths
                .into_iter()
                .filter_map(|p| p.into_path().ok())
                .map(|p| p.to_string_lossy().to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Native folder picker for conversion output.
#[tauri::command]
fn pick_output_dir(app: tauri::AppHandle) -> Option<String> {
    app.dialog()
        .file()
        .blocking_pick_folder()
        .and_then(|p| p.into_path().ok())
        .map(|p| p.to_string_lossy().to_string())
}

/// Convert PDFs at real filesystem paths; writes `<stem>.tei.xml` and
/// `<stem>.md` into `out_dir` (defaults to `markers-out/` beside the first PDF).
#[tauri::command]
async fn convert_pdfs(
    paths: Vec<String>,
    out_dir: Option<String>,
) -> Result<Vec<FileOutcome>, String> {
    if paths.is_empty() {
        return Ok(Vec::new());
    }
    let out = match out_dir {
        Some(d) => PathBuf::from(d),
        None => default_out_dir(&paths)?,
    };
    std::fs::create_dir_all(&out).map_err(|e| format!("create {}: {e}", out.display()))?;

    let client = http_client().await?;
    ensure_grobid().await?;

    let mut results = Vec::with_capacity(paths.len());
    for raw in &paths {
        let pdf = PathBuf::from(raw);
        let name = pdf
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| raw.clone());
        match convert_one(&client, &pdf, &out).await {
            Ok((tei, md)) => results.push(FileOutcome {
                name,
                ok: true,
                error: None,
                tei_path: Some(tei.to_string_lossy().to_string()),
                md_path: Some(md.to_string_lossy().to_string()),
            }),
            Err(e) => results.push(FileOutcome {
                name,
                ok: false,
                error: Some(e),
                tei_path: None,
                md_path: None,
            }),
        }
    }
    grobid_client::stop_local_service();
    Ok(results)
}

/// Convert uploaded PDFs (bytes from a file input or drag-drop without
/// paths). Outputs are written to a temp dir and returned inline.
#[tauri::command]
async fn convert_uploads(files: Vec<UploadFile>) -> Result<Vec<UploadOutcome>, String> {
    if files.is_empty() {
        return Ok(Vec::new());
    }
    let client = http_client().await?;
    ensure_grobid().await?;

    let tmp = std::env::temp_dir().join(format!("markers-desktop-{}", std::process::id()));
    let pdf_dir = tmp.join("pdf");
    let out_dir = tmp.join("out");
    std::fs::create_dir_all(&pdf_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;

    let mut results = Vec::with_capacity(files.len());
    for f in files {
        let name = if f.name.to_lowercase().ends_with(".pdf") {
            f.name.clone()
        } else {
            format!("{}.pdf", f.name)
        };
        let pdf_path = pdf_dir.join(&name);
        let outcome = async {
            std::fs::write(&pdf_path, &f.data).map_err(|e| e.to_string())?;
            let (tei, md) = convert_one(&client, &pdf_path, &out_dir).await?;
            let tei_xml = std::fs::read_to_string(&tei).map_err(|e| e.to_string())?;
            let md_text = std::fs::read_to_string(&md).map_err(|e| e.to_string())?;
            Ok::<_, String>((tei_xml, md_text))
        }
        .await;
        match outcome {
            Ok((tei_xml, md)) => results.push(UploadOutcome {
                name,
                ok: true,
                error: None,
                tei_xml: Some(tei_xml),
                md: Some(md),
            }),
            Err(e) => results.push(UploadOutcome {
                name,
                ok: false,
                error: Some(e),
                tei_xml: None,
                md: None,
            }),
        }
    }
    let _ = std::fs::remove_dir_all(&tmp);
    grobid_client::stop_local_service();
    Ok(results)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            pick_pdfs,
            pick_output_dir,
            convert_pdfs,
            convert_uploads
        ])
        .run(tauri::generate_context!())
        .expect("error while running markers desktop");
}
