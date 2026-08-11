# BFG Repo Cleaner Native

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust Edition](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)

A high-performance, native **Rust** implementation of the renowned [BFG Repo Cleaner](https://github.com/rtyley/bfg-repo-cleaner) (originally written in Scala/Java).

**BFG Repo Cleaner Native** is engineered to clean up Git repository history by removing unwanted large files, scrubbing leaked credentials/secrets, and purging sensitive blobs across past commits with native execution speeds, rich ANSI terminal output, and minimal memory overhead.

---

## 🇧🇷 Resumo em Português / Portuguese Summary

O **BFG Repo Cleaner Native** é uma ferramenta de alta performance desenvolvida em **Rust** sob licença **GPLv3** para expurgar dados sensíveis (senhas, chaves de API, credenciais) e remover arquivos grandes do histórico de commits do Git. Ela acessa diretamente o banco de dados de objetos do Git (`git2`/`libgit2`), reescrevendo a árvore de commits e atualizando referências com interface CLI colorida, confortável e intuitiva.

---

## ⚡ Key Features

- 🚀 **Direct Git ODB Scanning**: Ultra-fast low-level access to the Git Object Database (`git2` / `libgit2`).
- 🎨 **Comfortable & Colorful CLI**: Vibrant ANSI terminal output with visual banners, progress indicators, and formatted summary cards.
- 📦 **Large Blob Removal**: Filter and eliminate blobs exceeding a specified size threshold (e.g., >10MB).
- 🔑 **Secrets Redaction (Regex)**: Replace leaked secrets, tokens, and credentials in file content with `***REDACTED***`.
- 📁 **File & Directory Stripping**: Match and erase unwanted files or folders by glob patterns (`*.zip`, `node_modules`, `id_rsa`).
- 🛡️ **HEAD Protection**: Protects blobs present in the latest commit (`HEAD`) by default to prevent accidental breakage of current working state.
- 🔍 **Automatic Repository Detection**: Detects if executed inside a valid Git repository or prompts interactively.
- 💻 **Flexible CLI Interface**: Full command-line argument support powered by `clap`.

---

## 🛠️ Prerequisites & Compilation

### Requirements
- [Rust & Cargo](https://www.rust-lang.org/) (2021/2024 Edition)
- `libgit2` / `openssl` native dependencies (handled automatically by Cargo)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/lucasmrthomaz/bfg-repo-cleaner-thz.git
cd bfg-repo-cleaner-thz

# Build in release mode
cargo build --release
```

The optimized binary will be created at `target/release/bfg-repo-cleaner-native`.

---

## 📖 Command Line Usage

### 1. Run inside current Git repository
If your terminal is inside the root folder of a Git repository:

```bash
cargo run --release
```

### 2. Strip files larger than a threshold
Remove all historical files larger than 10MB (10,485,760 bytes):

```bash
cargo run --release -- /path/to/repo -b 10485760
```

### 3. Redact secrets using regular expressions
Search and replace API key formats across all commits:

```bash
# Redact Google API keys
cargo run --release -- /path/to/repo -r "AIzaSy[A-Za-z0-9-_]{35}"

# Redact AWS Access Keys
cargo run --release -- /path/to/repo -r "AKIA[0-9A-Z]{16}"
```

### 4. Delete specific files or folders by pattern
Delete sensitive key files or large binary archives across history:

```bash
# Delete private keys or logs
cargo run --release -- /path/to/repo -D "id_rsa"
cargo run --release -- /path/to/repo -D "*.log"

# Delete folder hierarchies
cargo run --release -- /path/to/repo --delete-folders "node_modules"
```

### 5. Strip HEAD blobs (Disable Protection)
By default, files currently present in `HEAD` are protected. To override protection and modify `HEAD` as well:

```bash
cargo run --release -- /path/to/repo -b 10485760 --no-blob-protection
```

### 6. View CLI Options & Help

```bash
cargo run --release -- --help
```

---

## 🛡️ Post-Clean Steps & Git Garbage Collection

After cleaning a repository history, run the following Git commands inside the target repository to permanently prune dangling objects and reclaim disk space:

```bash
# Expire reflog entries
git reflog expire --expire=now --all

# Prune loose objects and repack database
git gc --prune=now --aggressive

# Force-push updated branches and tags to remote (Use caution!)
git push origin --force --all
git push origin --force --tags
```

> [!WARNING]
> Rewriting Git history alters commit SHAs. Coordinate with your team before force-pushing rewritten branches to remote repositories.

---

## 🏗️ Architecture & Project Structure

```
bfg-repo-cleaner-native/
├── .github/
│   └── workflows/
│       └── ci.yml          # GitHub Actions CI workflow (Check, Test, Clippy, Format)
├── src/
│   ├── main.rs             # Application entrypoint, banner logging & colored summary
│   ├── cli.rs              # CLI parser (`clap`) with colored styles & path resolution
│   ├── engine.rs           # Git ODB scanner & topological history rewriter
│   ├── filter.rs           # Glob file matching & Regex content redactor
│   └── models.rs           # Data structures (`CleanerOptions`, `ExecutionSummary`)
├── Cargo.toml              # Package configuration & metadata
├── LICENSE                 # GNU General Public License v3.0 (GPLv3)
└── README.md               # Project documentation
```

---

## 🧪 Testing & Code Quality

Run automated unit tests, lint checks, and formatting verification:

```bash
# Run unit tests
cargo test

# Run Clippy linter
cargo clippy -- -D warnings

# Check code formatting
cargo fmt --check
```

---

## 📄 License & Credits

- **License**: Licensed under the [GNU General Public License v3.0 (GPLv3)](LICENSE).
- **Authors**: Lucas M. R. Thomaz & Roberto Tyley.
- **Original Scala BFG Repo Cleaner**: Created by [Roberto Tyley](https://github.com/rtyley) at [rtyley/bfg-repo-cleaner](https://github.com/rtyley).
