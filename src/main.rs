//! BFG Repo Cleaner Native
//!
//! High-performance native Rust implementation of the BFG Repo Cleaner.
//! Designed for removing large files, sensitive data, and secrets from Git repository history.
//!
//! Implementação nativa de alta performance em Rust do BFG Repo Cleaner.
//! Projetado para remover arquivos grandes e expurgar dados sensíveis/segredos do histórico Git.
//!
//! Original Scala project: https://github.com/rtyley/bfg-repo-cleaner

mod cli;
mod engine;
mod filter;
mod models;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, resolve_repo_path};
use colored::Colorize;
use engine::BfgEngine;
use models::{CleanerOptions, ExecutionSummary};

fn print_banner() {
    println!(
        "{}",
        "┌──────────────────────────────────────────────────────────┐".bright_cyan()
    );
    println!(
        "{}",
        "│         ⚡ BFG REPO CLEANER NATIVE (RUST) ⚡              │"
            .bright_yellow()
            .bold()
    );
    println!(
        "{}",
        "│      Scrub secrets & large files from Git history        │".cyan()
    );
    println!(
        "{}",
        "└──────────────────────────────────────────────────────────┘".bright_cyan()
    );
    println!();
}

fn print_summary_card(summary: &ExecutionSummary) {
    println!();
    println!(
        "{}",
        "┌──────────────────────────────────────────────────────────┐".bright_green()
    );
    println!(
        "{}",
        "│           📊 EXECUTION SUMMARY / RESUMO FINAL            │"
            .bright_green()
            .bold()
    );
    println!(
        "{}",
        "├──────────────────────────────────────────────────────────┤".bright_green()
    );
    println!(
        "│  {:30} : {:>18}  │",
        "Total Blobs Scanned".white(),
        summary.total_blobs_scanned.to_string().yellow().bold()
    );
    println!(
        "│  {:30} : {:>18}  │",
        "Blobs Stripped/Removed".white(),
        summary.blobs_removed.to_string().red().bold()
    );
    println!(
        "│  {:30} : {:>18}  │",
        "Secrets Redacted".white(),
        summary.secrets_redacted.to_string().magenta().bold()
    );
    println!(
        "│  {:30} : {:>18}  │",
        "Commits Rewritten".white(),
        summary.total_commits_rewritten.to_string().cyan().bold()
    );
    println!(
        "│  {:30} : {:>18}  │",
        "Trees Rewritten".white(),
        summary.total_trees_rewritten.to_string().cyan()
    );
    println!(
        "│  {:30} : {:>18}  │",
        "Refs Updated".white(),
        summary.total_refs_updated.to_string().blue().bold()
    );
    println!(
        "│  {:30} : {:>18}  │",
        "Execution Time (ms)".white(),
        format!("{} ms", summary.execution_time_ms).bright_yellow()
    );
    println!(
        "{}",
        "└──────────────────────────────────────────────────────────┘".bright_green()
    );
    println!();
}

fn main() -> Result<()> {
    print_banner();

    let cli = Cli::parse();
    let repo_path = resolve_repo_path(cli.repo_path)?;

    let options = CleanerOptions {
        repo_path,
        max_file_size_bytes: cli.max_file_size_bytes,
        delete_files: cli.delete_files,
        delete_folders: cli.delete_folders,
        regex_pattern: cli.regex_pattern,
        protect_blobs_from: cli
            .protect_blobs_from
            .unwrap_or_else(|| vec!["HEAD".to_string()]),
        no_blob_protection: cli.no_blob_protection,
        strip_blobs_with_ids: None,
    };

    let engine = BfgEngine::new(options)?;
    let summary = engine.execute()?;

    print_summary_card(&summary);

    Ok(())
}
