//! Data models and configuration structures for BFG Repo Cleaner Native.
//! Modelos de dados e estruturas de configuração para o BFG Repo Cleaner Native.

use git2::Oid;
use std::collections::HashSet;
use std::path::PathBuf;

/// Execution options passed to the BFG Engine.
/// Opções de execução passadas ao motor do BFG.
#[derive(Debug, Clone)]
pub struct CleanerOptions {
    /// Path to the target Git repository / Caminho do repositório Git alvo
    pub repo_path: PathBuf,
    /// Maximum allowed file size in bytes / Tamanho máximo de arquivo em bytes
    pub max_file_size_bytes: Option<usize>,
    /// Glob pattern for files to delete / Padrão glob de arquivos a deletar
    pub delete_files: Option<String>,
    /// Glob pattern for folders to delete / Padrão glob de pastas a deletar
    pub delete_folders: Option<String>,
    /// Regular expression pattern for scrubbing secrets / Regex para redação de segredos
    pub regex_pattern: Option<String>,
    /// Git revision references to protect from modification / Referências Git a proteger
    pub protect_blobs_from: Vec<String>,
    /// Whether to disable default protection of the latest revision / Desativar proteção de blobs no HEAD
    pub no_blob_protection: bool,
    /// Optional set of specific Object IDs to strip / OIDs específicos a remover
    pub strip_blobs_with_ids: Option<HashSet<Oid>>,
}

/// Statistics and metrics collected after completing repository cleaning.
/// Estatísticas e métricas coletadas após a execução da limpeza no repositório.
#[allow(dead_code)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExecutionSummary {
    /// Total number of Git blobs scanned / Total de blobs escaneados
    pub total_blobs_scanned: usize,
    /// Total number of blobs removed / Total de blobs removidos
    pub blobs_removed: usize,
    /// Total number of secret occurrences redacted / Total de segredos expurgados
    pub secrets_redacted: usize,
    /// Total number of commits rewritten / Total de commits reescritos
    pub total_commits_rewritten: usize,
    /// Total number of tree objects rewritten / Total de árvores reescritas
    pub total_trees_rewritten: usize,
    /// Total number of Git references (branches/tags) updated / Total de refs atualizadas
    pub total_refs_updated: usize,
    /// Total execution duration in milliseconds / Duração da execução em milissegundos
    pub execution_time_ms: u128,
}
