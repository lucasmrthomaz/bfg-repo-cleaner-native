use bfg_repo_cleaner_native::engine::BfgEngine;
use bfg_repo_cleaner_native::models::{CleanerOptions, ExecutionSummary};
use git2::Repository;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanerOptionsPayload {
    pub repo_path: String,
    pub max_file_size_bytes: Option<usize>,
    pub delete_files: Option<String>,
    pub delete_folders: Option<String>,
    pub regex_pattern: Option<String>,
    pub protect_blobs_from: Option<Vec<String>>,
    pub no_blob_protection: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoValidationResult {
    pub valid: bool,
    pub is_bare: bool,
    pub head_ref: Option<String>,
    pub message: String,
}

#[tauri::command]
fn select_repository_folder() -> Option<String> {
    let folder = rfd::FileDialog::new()
        .set_title("Select Git Repository Directory / Selecionar Diretório do Repositório Git")
        .pick_folder()?;

    Some(folder.to_string_lossy().to_string())
}

#[tauri::command]
fn validate_repository(repo_path: String) -> RepoValidationResult {
    let path = PathBuf::from(&repo_path);
    if !path.exists() {
        return RepoValidationResult {
            valid: false,
            is_bare: false,
            head_ref: None,
            message: "Directory does not exist. / O diretório informado não existe.".to_string(),
        };
    }

    match Repository::open(&path) {
        Ok(repo) => {
            let is_bare = repo.is_bare();
            let head_ref = repo
                .head()
                .ok()
                .and_then(|r| r.shorthand().map(|s| s.to_string()));

            RepoValidationResult {
                valid: true,
                is_bare,
                head_ref,
                message: "Valid Git repository detected! / Repositório Git válido detectado!"
                    .to_string(),
            }
        }
        Err(err) => RepoValidationResult {
            valid: false,
            is_bare: false,
            head_ref: None,
            message: format!(
                "Not a valid Git repository: {} / Não é um repositório Git válido.",
                err.message()
            ),
        },
    }
}

#[tauri::command]
fn execute_cleaner(payload: CleanerOptionsPayload) -> Result<ExecutionSummary, String> {
    let repo_path = PathBuf::from(&payload.repo_path);
    if !repo_path.exists() {
        return Err("Repository path does not exist. / Caminho do repositório não existe.".to_string());
    }

    let delete_files = payload
        .delete_files
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let delete_folders = payload
        .delete_folders
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let regex_pattern = payload
        .regex_pattern
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let protect_blobs_from = payload
        .protect_blobs_from
        .unwrap_or_else(|| vec!["HEAD".to_string()]);

    let options = CleanerOptions {
        repo_path,
        max_file_size_bytes: payload.max_file_size_bytes,
        delete_files,
        delete_folders,
        regex_pattern,
        protect_blobs_from,
        no_blob_protection: payload.no_blob_protection,
        strip_blobs_with_ids: None,
    };

    let engine = BfgEngine::new(options).map_err(|e| e.to_string())?;
    let summary = engine.execute().map_err(|e| e.to_string())?;

    Ok(summary)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    {
        if std::env::var("GDK_BACKEND").is_err() {
            unsafe {
                std::env::set_var("GDK_BACKEND", "x11,wayland");
            }
        }
        if std::env::var("WEBKIT_DISABLE_COMPOSITING_MODE").is_err() {
            unsafe {
                std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
            }
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            select_repository_folder,
            validate_repository,
            execute_cleaner
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
