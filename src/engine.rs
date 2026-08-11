//! Core BFG Engine for scanning Git Object Database (ODB) and rewriting commit history.
//! Motor principal do BFG para varredura do Git ODB e reescrita do histórico de commits.

#![allow(clippy::collapsible_if)]

use crate::filter::{FileMatcher, Redactor};
use crate::models::{CleanerOptions, ExecutionSummary};
use anyhow::{Context, Result};
use colored::Colorize;
use git2::{ObjectType, Oid, Repository, Tree};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

/// Core execution engine for Git history cleaning.
/// Motor principal de execução para limpeza do histórico Git.
pub struct BfgEngine {
    options: CleanerOptions,
    repo: Repository,
}

impl BfgEngine {
    /// Initializes the BFG Engine with target options and opens the Git repository.
    /// Inicializa o motor do BFG com as opções e abre o repositório Git.
    pub fn new(options: CleanerOptions) -> Result<Self> {
        let repo = Repository::open(&options.repo_path).with_context(|| {
            format!(
                "Failed to open Git repository at / Falha ao abrir repositório em {:?}",
                options.repo_path
            )
        })?;
        Ok(Self { options, repo })
    }

    /// Executes the full cleaning pipeline over the Git history.
    /// Executa o pipeline completo de limpeza no histórico Git.
    pub fn execute(&self) -> Result<ExecutionSummary> {
        let start = Instant::now();

        // Step 1: Collect protected blobs (e.g., HEAD revision)
        // Passo 1: Mapear os blobs protegidos (ex: presentes no HEAD)
        let protected_blobs = self.collect_protected_blobs()?;
        if !protected_blobs.is_empty() {
            println!(
                "{} {} {}",
                "🔒 Protected blobs in HEAD / Blobs protegidos no HEAD:"
                    .bright_blue()
                    .bold(),
                protected_blobs.len().to_string().yellow().bold(),
                "blobs preserved".dimmed()
            );
        }

        // Step 2: Prepare file, folder, and secret redactor matchers
        // Passo 2: Preparar matchers de arquivo, diretório e redator de segredos
        let file_matcher = match &self.options.delete_files {
            Some(pattern) => Some(FileMatcher::new(pattern)?),
            None => None,
        };
        let folder_matcher = match &self.options.delete_folders {
            Some(pattern) => Some(FileMatcher::new(pattern)?),
            None => None,
        };
        let redactor = Redactor::new(self.options.regex_pattern.as_deref())?;

        // Step 3: Scan ODB to identify unwanted blobs (by size or OID)
        // Passo 3: Varrer ODB e identificar blobs indesejados (por tamanho ou ID)
        let odb = self.repo.odb()?;
        let mut scanned = 0;
        let mut bad_blobs = HashSet::new();

        if let Some(strip_ids) = &self.options.strip_blobs_with_ids {
            for oid in strip_ids {
                if !protected_blobs.contains(oid) {
                    bad_blobs.insert(*oid);
                }
            }
        }

        odb.foreach(|oid| {
            if let Ok((size, kind)) = odb.read_header(*oid) {
                if kind == ObjectType::Blob {
                    scanned += 1;
                    if let Some(max_bytes) = self.options.max_file_size_bytes {
                        if size > max_bytes && !protected_blobs.contains(oid) {
                            bad_blobs.insert(*oid);
                        }
                    }
                }
            }
            true
        })?;

        println!(
            "{} {} {}",
            "🔍 Git ODB Scanned / Blobs analisados:".cyan().bold(),
            scanned.to_string().yellow().bold(),
            "objects".dimmed()
        );

        if !bad_blobs.is_empty() {
            println!(
                "{} {}",
                "⚡ Blobs flagged for stripping / Blobs identificados para remoção:"
                    .bright_magenta()
                    .bold(),
                bad_blobs.len().to_string().red().bold()
            );
        }

        // Step 4: Early return if no cleaning filters are active or needed
        // Passo 4: Retorno rápido se nenhuma alteração for necessária
        let has_work = !bad_blobs.is_empty()
            || file_matcher.is_some()
            || folder_matcher.is_some()
            || redactor.has_pattern();

        if !has_work {
            println!(
                "{}",
                "✨ No repository changes required / Nenhuma alteração necessária."
                    .bright_green()
                    .bold()
            );
            return Ok(ExecutionSummary {
                total_blobs_scanned: scanned,
                blobs_removed: 0,
                secrets_redacted: 0,
                total_commits_rewritten: 0,
                total_trees_rewritten: 0,
                total_refs_updated: 0,
                execution_time_ms: start.elapsed().as_millis(),
            });
        }

        // Step 5: Rewrite Commit and Tree history in Topological / Reverse order
        // Passo 5: Reescrita do histórico de commits e árvores (Ordem Topológica)
        println!(
            "{}",
            "🚀 Rewriting repository history... / Reescrevendo histórico de commits..."
                .bright_cyan()
                .bold()
        );

        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_glob("refs/*")?;
        revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::REVERSE)?;

        let mut rewritten_commits: HashMap<Oid, Oid> = HashMap::new();
        let mut rewritten_trees: HashMap<Oid, Oid> = HashMap::new();
        let mut commits_count = 0;
        let mut blobs_removed = 0;
        let mut secrets_redacted = 0;

        for rev in revwalk {
            let commit_oid = match rev {
                Ok(id) => id,
                Err(_) => continue,
            };

            let commit = match self.repo.find_commit(commit_oid) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let tree = commit.tree()?;
            let new_tree_oid = self.rewrite_tree(
                &tree,
                &bad_blobs,
                &file_matcher,
                &folder_matcher,
                &redactor,
                &mut rewritten_trees,
                &mut blobs_removed,
                &mut secrets_redacted,
            )?;

            // Map updated parent commits / Mapear pais atualizados do commit
            let new_parents: Vec<git2::Commit> = commit
                .parents()
                .map(|p| {
                    let parent_oid = p.id();
                    let new_parent_oid = rewritten_commits
                        .get(&parent_oid)
                        .copied()
                        .unwrap_or(parent_oid);
                    self.repo.find_commit(new_parent_oid)
                })
                .collect::<Result<Vec<_>, _>>()?;

            let parent_refs: Vec<&git2::Commit> = new_parents.iter().collect();

            // Re-create commit with modified tree & parents / Recriar commit com nova árvore e pais
            let new_commit_oid = self.repo.commit(
                None, // References updated in step 6 / Refs atualizadas no passo 6
                &commit.author(),
                &commit.committer(),
                commit.message().unwrap_or(""),
                &self.repo.find_tree(new_tree_oid)?,
                &parent_refs,
            )?;

            rewritten_commits.insert(commit_oid, new_commit_oid);
            commits_count += 1;
        }

        // Step 6: Update Git References (Branches & Tags)
        // Passo 6: Atualizar Referências Git (Branches e Tags)
        let mut refs_updated = 0;
        let references = self.repo.references()?;
        for reference in references {
            let mut reference = match reference {
                Ok(r) => r,
                Err(_) => continue,
            };

            if reference.is_branch() || reference.is_tag() {
                if let Some(target) = reference.target() {
                    if let Some(&new_target) = rewritten_commits.get(&target) {
                        if target != new_target {
                            reference.set_target(new_target, "BFG Repo Cleaner native rewrite")?;
                            refs_updated += 1;
                        }
                    }
                }
            }
        }

        Ok(ExecutionSummary {
            total_blobs_scanned: scanned,
            blobs_removed,
            secrets_redacted,
            total_commits_rewritten: commits_count,
            total_trees_rewritten: rewritten_trees.len(),
            total_refs_updated: refs_updated,
            execution_time_ms: start.elapsed().as_millis(),
        })
    }

    /// Recursively traverses Git tree to collect protected blob OIDs.
    /// Percorre recursivamente a árvore Git e coleta OIDs de blobs protegidos.
    fn collect_protected_blobs(&self) -> Result<HashSet<Oid>> {
        if self.options.no_blob_protection {
            return Ok(HashSet::new());
        }

        let mut protected = HashSet::new();
        for rev_spec in &self.options.protect_blobs_from {
            if let Ok(obj) = self.repo.revparse_single(rev_spec) {
                if let Ok(commit) = obj.peel_to_commit() {
                    if let Ok(tree) = commit.tree() {
                        self.collect_tree_blobs(&tree, &mut protected)?;
                    }
                }
            }
        }

        Ok(protected)
    }

    fn collect_tree_blobs(&self, tree: &Tree, protected: &mut HashSet<Oid>) -> Result<()> {
        for entry in tree.iter() {
            match entry.kind() {
                Some(ObjectType::Blob) => {
                    protected.insert(entry.id());
                }
                Some(ObjectType::Tree) => {
                    if let Ok(sub_tree) = self.repo.find_tree(entry.id()) {
                        self.collect_tree_blobs(&sub_tree, protected)?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Recursively rewrites Git tree filtering files, folders, and redacting secret contents.
    /// Re-escreve recursivamente a árvore Git filtrando arquivos, pastas e redigindo segredos.
    #[allow(clippy::too_many_arguments)]
    fn rewrite_tree(
        &self,
        tree: &Tree,
        bad_blobs: &HashSet<Oid>,
        file_matcher: &Option<FileMatcher>,
        folder_matcher: &Option<FileMatcher>,
        redactor: &Redactor,
        cache: &mut HashMap<Oid, Oid>,
        blobs_removed: &mut usize,
        secrets_redacted: &mut usize,
    ) -> Result<Oid> {
        if let Some(&cached_oid) = cache.get(&tree.id()) {
            return Ok(cached_oid);
        }

        let mut builder = self.repo.treebuilder(None)?;

        for entry in tree.iter() {
            let name = match entry.name() {
                Some(n) => n,
                None => continue,
            };

            match entry.kind() {
                Some(ObjectType::Tree) => {
                    if let Some(matcher) = folder_matcher {
                        if matcher.is_match(name) {
                            continue; // Skip deleted folder / Ignorar pasta removida
                        }
                    }

                    let sub_tree = self.repo.find_tree(entry.id())?;
                    let new_sub_tree_oid = self.rewrite_tree(
                        &sub_tree,
                        bad_blobs,
                        file_matcher,
                        folder_matcher,
                        redactor,
                        cache,
                        blobs_removed,
                        secrets_redacted,
                    )?;

                    builder.insert(name, new_sub_tree_oid, entry.filemode())?;
                }
                Some(ObjectType::Blob) => {
                    if let Some(matcher) = file_matcher {
                        if matcher.is_match(name) {
                            *blobs_removed += 1;
                            continue; // Skip deleted file / Ignorar arquivo removido
                        }
                    }

                    if bad_blobs.contains(&entry.id()) {
                        *blobs_removed += 1;
                        continue; // Skip oversized or unwanted blob / Ignorar blob grande removido
                    }

                    if redactor.has_pattern() {
                        if let Ok(blob) = self.repo.find_blob(entry.id()) {
                            if let Ok(content_str) = std::str::from_utf8(blob.content()) {
                                let redacted_content = redactor.redact(content_str);
                                if redacted_content != content_str {
                                    let new_blob_oid =
                                        self.repo.blob(redacted_content.as_bytes())?;
                                    builder.insert(name, new_blob_oid, entry.filemode())?;
                                    *secrets_redacted += 1;
                                    continue;
                                }
                            }
                        }
                    }

                    // Keep original blob / Manter blob original
                    builder.insert(name, entry.id(), entry.filemode())?;
                }
                _ => {
                    builder.insert(name, entry.id(), entry.filemode())?;
                }
            }
        }

        let new_tree_oid = builder.write()?;
        cache.insert(tree.id(), new_tree_oid);
        Ok(new_tree_oid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_engine_init_and_execution_dry_run() {
        let temp_dir = std::env::temp_dir().join(format!(
            "bfg_test_repo_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let repo = Repository::init(&temp_dir).unwrap();

        // Create a commit
        let file_path = temp_dir.join("hello.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello World!").unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("hello.txt")).unwrap();
        let oid = index.write_tree().unwrap();

        let signature = repo.signature().unwrap();
        let tree = repo.find_tree(oid).unwrap();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        )
        .unwrap();

        let options = CleanerOptions {
            repo_path: temp_dir.clone(),
            max_file_size_bytes: Some(10485760),
            delete_files: None,
            delete_folders: None,
            regex_pattern: None,
            protect_blobs_from: vec!["HEAD".to_string()],
            no_blob_protection: false,
            strip_blobs_with_ids: None,
        };

        let engine = BfgEngine::new(options).unwrap();
        let summary = engine.execute().unwrap();

        assert_eq!(summary.blobs_removed, 0);
        assert_eq!(summary.secrets_redacted, 0);

        // Cleanup
        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
