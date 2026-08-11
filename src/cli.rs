//! Command Line Interface (CLI) parsing and repository path resolution.
//! Módulo responsável por parsing de linha de comando (CLI) e resolução de caminhos.

use anyhow::{Context, Result, bail};
use clap::Parser;
use clap::builder::styling::{AnsiColor, Effects, Styles};
use colored::Colorize;
use git2::Repository;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;

/// Custom color styling for Clap CLI help rendering.
/// Estilo de cores customizado para a ajuda da CLI.
fn cli_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::BrightYellow.on_default() | Effects::BOLD)
        .usage(AnsiColor::BrightGreen.on_default() | Effects::BOLD)
        .literal(AnsiColor::BrightCyan.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Cyan.on_default())
        .error(AnsiColor::BrightRed.on_default() | Effects::BOLD)
}

/// Command Line Interface (CLI) options for BFG Repo Cleaner Native.
/// Opções da interface de linha de comando (CLI) para o BFG Repo Cleaner Native.
#[derive(Parser, Debug)]
#[command(
    name = "bfg-repo-cleaner-native",
    version,
    about = "High-performance native Rust tool for scrubbing secrets and large files from Git history / Limpeza de repositórios Git",
    styles = cli_styles()
)]
pub struct Cli {
    /// Path to the target Git repository (optional if run inside a Git repo) / Caminho do repositório Git
    #[arg(value_name = "REPO_PATH")]
    pub repo_path: Option<PathBuf>,

    /// Strip blobs larger than specified size in bytes (e.g., 10485760 for 10MB) / Tamanho máximo em bytes
    #[arg(short = 'b', long = "strip-blobs-bigger-than")]
    pub max_file_size_bytes: Option<usize>,

    /// Delete files matching specified glob pattern (e.g., '*.zip', 'id_rsa') / Apagar arquivos por padrão glob
    #[arg(short = 'D', long = "delete-files")]
    pub delete_files: Option<String>,

    /// Delete folders matching specified glob pattern (e.g., '.svn', 'node_modules') / Apagar pastas por padrão glob
    #[arg(long = "delete-folders")]
    pub delete_folders: Option<String>,

    /// Regex pattern for replacing secrets with ***REDACTED*** / Regex para expurgo de segredos
    #[arg(short = 'r', long = "regex")]
    pub regex_pattern: Option<String>,

    /// Protect blobs present in specified revisions (default: "HEAD") / Proteger blobs das revisões especificadas
    #[arg(short = 'p', long = "protect-blobs-from", value_delimiter = ',')]
    pub protect_blobs_from: Option<Vec<String>>,

    /// Allow modifying blobs in the latest revision (HEAD) / Desativar proteção de blobs no HEAD
    #[arg(long = "no-blob-protection")]
    pub no_blob_protection: bool,
}

/// Resolves the Git repository path from CLI arguments, current working directory, or interactive user prompt.
/// Resolve o caminho do repositório Git com base nos argumentos, diretório atual ou entrada interativa.
pub fn resolve_repo_path(provided_path: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = provided_path {
        if !path.exists() {
            bail!(
                "{} '{}' {}",
                "✖ Error: The provided path".red().bold(),
                path.display().to_string().yellow(),
                "does not exist. / O caminho informado não existe.".red()
            );
        }
        return Ok(path);
    }

    let current_dir = env::current_dir()
        .context("Failed to determine current directory / Falha ao determinar o diretório atual")?;

    if Repository::open(&current_dir).is_ok() {
        println!(
            "{} {} {}",
            "✔ Git repository detected:".bright_green().bold(),
            current_dir.display().to_string().cyan().underline(),
            "(using current directory)".dimmed()
        );
        return Ok(current_dir);
    }

    print!(
        "{} ",
        "📂 Enter target Git repository path:"
            .bright_yellow()
            .bold()
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();

    if trimmed.is_empty() {
        bail!(
            "{}",
            "✖ No path provided and current directory is not a valid Git repository / Nenhum caminho informado."
                .bright_red()
                .bold()
        );
    }

    let path = PathBuf::from(trimmed);
    if !path.exists() {
        bail!(
            "{} '{}' {}",
            "✖ Error: The provided path".red().bold(),
            path.display().to_string().yellow(),
            "does not exist. / O caminho informado não existe.".red()
        );
    }

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_argument_parsing() {
        let args = vec![
            "bfg-repo-cleaner-native",
            "/tmp/test-repo",
            "-b",
            "10485760",
            "-r",
            "AKIA[0-9A-Z]{16}",
            "--no-blob-protection",
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.repo_path, Some(PathBuf::from("/tmp/test-repo")));
        assert_eq!(cli.max_file_size_bytes, Some(10485760));
        assert_eq!(cli.regex_pattern, Some("AKIA[0-9A-Z]{16}".to_string()));
        assert!(cli.no_blob_protection);
    }
}
