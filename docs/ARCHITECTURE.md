# Dotf Architecture

## 概要

Dotfは、Rustのベストプラクティスに従って設計されたdotfile管理のためのコマンドラインツールです。
テスタビリティ、保守性、拡張性を重視したアーキテクチャを採用しています。

## プロジェクト構造

```
dotf/
├── Cargo.toml
├── README.md
├── docs/
│   ├── SPECIFICATION.md
│   └── ARCHITECTURE.md
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── init.rs
│   │   │   ├── install.rs
│   │   │   ├── status.rs
│   │   │   ├── sync.rs
│   │   │   ├── symlinks.rs
│   │   │   └── config.rs
│   │   └── args.rs
│   ├── core/
│   │   ├── mod.rs
│   │   ├── config/
│   │   │   ├── mod.rs
│   │   │   ├── dotf_config.rs
│   │   │   ├── settings.rs
│   │   │   └── validation.rs
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   ├── git.rs
│   │   │   └── manager.rs
│   │   ├── symlinks/
│   │   │   ├── mod.rs
│   │   │   ├── manager.rs
│   │   │   ├── conflict.rs
│   │   │   └── backup.rs
│   │   ├── scripts/
│   │   │   ├── mod.rs
│   │   │   ├── executor.rs
│   │   │   └── platform.rs
│   │   └── filesystem/
│   │       ├── mod.rs
│   │       ├── operations.rs
│   │       └── paths.rs
│   ├── services/
│   │   ├── mod.rs
│   │   ├── init_service.rs
│   │   ├── install_service.rs
│   │   ├── status_service.rs
│   │   ├── sync_service.rs
│   │   └── config_service.rs
│   ├── traits/
│   │   ├── mod.rs
│   │   ├── repository.rs
│   │   ├── filesystem.rs
│   │   ├── script_executor.rs
│   │   └── prompt.rs
│   ├── error/
│   │   ├── mod.rs
│   │   └── types.rs
│   └── utils/
│       ├── mod.rs
│       ├── platform.rs
│       ├── prompt.rs
│       └── output.rs
└── tests/
    ├── integration/
    │   ├── mod.rs
    │   ├── init_tests.rs
    │   ├── install_tests.rs
    │   └── sync_tests.rs
    ├── fixtures/
    │   ├── sample_dotf.toml
    │   └── test_repo/
    └── common/
        ├── mod.rs
        ├── mocks.rs
        └── test_utils.rs
```

## アーキテクチャレイヤー

### 1. CLI Layer (`src/cli/`)

**責務**: コマンドライン引数の解析とユーザーインターフェース

#### `args.rs`
```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dotf")]
#[command(about = "A modern dotfile installation tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Init {
        #[arg(long)]
        repo: Option<String>,
    },
    Install {
        #[command(subcommand)]
        target: InstallTarget,
    },
    Status {
        #[arg(long)]
        quiet: bool,
    },
    Sync {
        #[arg(long)]
        force: bool,
    },
    Symlinks {
        #[command(subcommand)]
        action: Option<SymlinksAction>,
    },
    Config {
        #[arg(long)]
        repo: bool,
        #[arg(long)]
        edit: bool,
    },
}

#[derive(Subcommand)]
pub enum InstallTarget {
    Deps,
    Config,
    Custom { name: String },
}

#[derive(Subcommand)]
pub enum SymlinksAction {
    Restore {
        #[arg(long)]
        list: bool,
        #[arg(long)]
        all: bool,
        filepath: Option<String>,
    },
}
```

### 2. Service Layer (`src/services/`)

**責務**: ビジネスロジックの実装とコマンド実行

#### `init_service.rs`
```rust
use crate::core::config::DotfConfig;
use crate::core::repository::RepositoryManager;
use crate::traits::{Repository, Prompt, FileSystem};
use crate::error::DotfResult;

pub struct InitService<R, F, P>
where
    R: Repository,
    F: FileSystem,
    P: Prompt,
{
    repository: R,
    filesystem: F,
    prompt: P,
}

impl<R, F, P> InitService<R, F, P>
where
    R: Repository,
    F: FileSystem,
    P: Prompt,
{
    pub fn new(repository: R, filesystem: F, prompt: P) -> Self {
        Self {
            repository,
            filesystem,
            prompt,
        }
    }

    pub async fn init(&self, repo_url: Option<String>) -> DotfResult<()> {
        let url = match repo_url {
            Some(url) => url,
            None => self.prompt.ask_repository_url().await?,
        };

        // 1. リポジトリの検証
        self.repository.validate_remote(&url).await?;

        // 2. 設定ファイルの取得・検証
        let config = self.repository.fetch_config(&url).await?;
        DotfConfig::validate(&config)?;

        // 3. ローカル環境の構築
        self.filesystem.create_dotf_directory().await?;
        self.repository.clone(&url, &self.filesystem.dotf_repo_path()).await?;
        self.filesystem.create_settings(&url).await?;

        Ok(())
    }
}
```

### 3. Core Layer (`src/core/`)

**責務**: ドメインロジックとデータ構造

#### `config/dotf_config.rs`
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct DotfConfig {
    pub repo: RepoConfig,
    #[serde(default)]
    pub symlinks: HashMap<String, String>,
    #[serde(default)]
    pub scripts: ScriptsConfig,
    #[serde(default)]
    pub platform: PlatformConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RepoConfig {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ScriptsConfig {
    #[serde(default)]
    pub deps: DepsScripts,
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct DepsScripts {
    pub macos: Option<String>,
    pub linux: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct PlatformConfig {
    pub macos: Option<PlatformSymlinks>,
    pub linux: Option<PlatformSymlinks>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlatformSymlinks {
    pub symlinks: Vec<(String, String)>,
}
```

#### `symlinks/manager.rs`
```rust
use crate::traits::{FileSystem, Prompt};
use crate::core::symlinks::{ConflictResolver, BackupManager};
use crate::error::DotfResult;
use std::collections::HashMap;

pub struct SymlinkManager<F, P>
where
    F: FileSystem,
    P: Prompt,
{
    filesystem: F,
    prompt: P,
    backup_manager: BackupManager<F>,
    conflict_resolver: ConflictResolver<F, P>,
}

impl<F, P> SymlinkManager<F, P>
where
    F: FileSystem,
    P: Prompt,
{
    pub fn new(filesystem: F, prompt: P) -> Self {
        let backup_manager = BackupManager::new(filesystem.clone());
        let conflict_resolver = ConflictResolver::new(filesystem.clone(), prompt.clone());

        Self {
            filesystem,
            prompt,
            backup_manager,
            conflict_resolver,
        }
    }

    pub async fn create_symlinks(
        &self,
        symlinks: &HashMap<String, String>,
    ) -> DotfResult<Vec<SymlinkResult>> {
        let mut results = Vec::new();

        for (source, target) in symlinks {
            let result = self.create_symlink(source, target).await?;
            results.push(result);
        }

        Ok(results)
    }

    async fn create_symlink(&self, source: &str, target: &str) -> DotfResult<SymlinkResult> {
        // 競合チェック
        if self.filesystem.exists(target).await? {
            let action = self.conflict_resolver.resolve(target).await?;
            match action {
                ConflictAction::Backup => {
                    self.backup_manager.backup_file(target).await?;
                }
                ConflictAction::Abort => {
                    return Ok(SymlinkResult::Aborted(target.to_string()));
                }
            }
        }

        // シンボリックリンク作成
        self.filesystem.create_symlink(source, target).await?;
        Ok(SymlinkResult::Created(target.to_string()))
    }
}

#[derive(Debug)]
pub enum SymlinkResult {
    Created(String),
    Aborted(String),
    Error(String, crate::error::DotfError),
}

#[derive(Debug)]
pub enum ConflictAction {
    Backup,
    Abort,
}
```

### 4. Traits Layer (`src/traits/`)

**責務**: 依存性の抽象化とテスタビリティの提供

#### `repository.rs`
```rust
use async_trait::async_trait;
use crate::core::config::DotfConfig;
use crate::error::DotfResult;

#[async_trait]
pub trait Repository {
    async fn validate_remote(&self, url: &str) -> DotfResult<()>;
    async fn fetch_config(&self, url: &str) -> DotfResult<DotfConfig>;
    async fn clone(&self, url: &str, destination: &str) -> DotfResult<()>;
    async fn pull(&self, repo_path: &str) -> DotfResult<()>;
    async fn get_status(&self, repo_path: &str) -> DotfResult<RepositoryStatus>;
    async fn get_remote_url(&self, repo_path: &str) -> DotfResult<String>;
}

#[derive(Debug)]
pub struct RepositoryStatus {
    pub is_clean: bool,
    pub ahead_count: usize,
    pub behind_count: usize,
    pub current_branch: String,
}
```

#### `filesystem.rs`
```rust
use async_trait::async_trait;
use crate::error::DotfResult;

#[async_trait]
pub trait FileSystem {
    async fn exists(&self, path: &str) -> DotfResult<bool>;
    async fn create_dir_all(&self, path: &str) -> DotfResult<()>;
    async fn create_symlink(&self, source: &str, target: &str) -> DotfResult<()>;
    async fn remove_file(&self, path: &str) -> DotfResult<()>;
    async fn copy_file(&self, source: &str, target: &str) -> DotfResult<()>;
    async fn read_to_string(&self, path: &str) -> DotfResult<String>;
    async fn write(&self, path: &str, content: &str) -> DotfResult<()>;

    // Dotf特有のパス操作
    fn dotf_directory(&self) -> String;
    fn dotf_repo_path(&self) -> String;
    fn dotf_settings_path(&self) -> String;
    fn dotf_backup_path(&self) -> String;

    async fn create_dotf_directory(&self) -> DotfResult<()>;
    async fn create_settings(&self, repo_url: &str) -> DotfResult<()>;
}
```

#### `script_executor.rs`
```rust
use async_trait::async_trait;
use crate::error::DotfResult;

#[async_trait]
pub trait ScriptExecutor {
    async fn execute(&self, script_path: &str) -> DotfResult<ExecutionResult>;
    async fn has_permission(&self, script_path: &str) -> DotfResult<bool>;
    async fn make_executable(&self, script_path: &str) -> DotfResult<()>;
}

#[derive(Debug)]
pub struct ExecutionResult {
    pub success: bool,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}
```

#### `prompt.rs`
```rust
use async_trait::async_trait;
use crate::error::DotfResult;
use crate::core::symlinks::ConflictAction;

#[async_trait]
pub trait Prompt {
    async fn ask_repository_url(&self) -> DotfResult<String>;
    async fn ask_conflict_resolution(&self, path: &str) -> DotfResult<ConflictAction>;
    async fn confirm(&self, message: &str) -> DotfResult<bool>;
}
```

### 5. Error Handling (`src/error/`)

#### `types.rs`
```rust
use thiserror::Error;

pub type DotfResult<T> = Result<T, DotfError>;

#[derive(Error, Debug)]
pub enum DotfError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Git error: {0}")]
    Git(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Platform not supported: {0}")]
    UnsupportedPlatform(String),

    #[error("Script execution failed: {0}")]
    ScriptExecution(String),

    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Symlink error: {0}")]
    Symlink(String),

    #[error("User cancelled operation")]
    UserCancelled,
}
```

## 実装クラス

### Git Implementation

#### `core/repository/git.rs`
```rust
use crate::traits::Repository;
use crate::error::{DotfResult, DotfError};
use async_trait::async_trait;
use git2::{Repository as Git2Repository, RemoteCallbacks};

pub struct GitRepository;

#[async_trait]
impl Repository for GitRepository {
    async fn validate_remote(&self, url: &str) -> DotfResult<()> {
        // git ls-remote でリモートリポジトリの存在確認
        let output = tokio::process::Command::new("git")
            .args(&["ls-remote", "--exit-code", url])
            .output()
            .await
            .map_err(|e| DotfError::Repository(format!("Failed to validate remote: {}", e)))?;

        if !output.status.success() {
            return Err(DotfError::Repository(format!(
                "Invalid repository URL: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    async fn fetch_config(&self, url: &str) -> DotfResult<DotfConfig> {
        // 一時的にdotf.tomlを取得
        let temp_dir = tempfile::tempdir()
            .map_err(|e| DotfError::Io(e))?;

        // Sparse checkoutでdotf.tomlのみ取得
        // 実装詳細...

        Ok(config)
    }

    // 他のメソッド実装...
}
```

### FileSystem Implementation

#### `core/filesystem/operations.rs`
```rust
use crate::traits::FileSystem;
use crate::error::{DotfResult, DotfError};
use async_trait::async_trait;
use tokio::fs;
use std::path::Path;

pub struct RealFileSystem;

#[async_trait]
impl FileSystem for RealFileSystem {
    async fn exists(&self, path: &str) -> DotfResult<bool> {
        Ok(Path::new(path).exists())
    }

    async fn create_symlink(&self, source: &str, target: &str) -> DotfResult<()> {
        // ターゲットディレクトリの作成
        if let Some(parent) = Path::new(target).parent() {
            fs::create_dir_all(parent).await?;
        }

        #[cfg(unix)]
        {
            tokio::fs::symlink(source, target).await
                .map_err(|e| DotfError::Symlink(format!("Failed to create symlink: {}", e)))?;
        }

        #[cfg(windows)]
        {
            // Windows実装（未対応）
            return Err(DotfError::UnsupportedPlatform("Windows".to_string()));
        }

        Ok(())
    }

    fn dotf_directory(&self) -> String {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".dotf")
            .to_string_lossy()
            .to_string()
    }

    // 他のメソッド実装...
}
```

## テスト戦略

### Mock Implementations

#### `tests/common/mocks.rs`
```rust
use crate::traits::{Repository, FileSystem, Prompt, ScriptExecutor};
use crate::error::DotfResult;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct MockRepository {
    pub expectations: Arc<Mutex<MockRepositoryExpectations>>,
}

#[derive(Default)]
pub struct MockRepositoryExpectations {
    pub validate_calls: Vec<String>,
    pub clone_calls: Vec<(String, String)>,
    pub should_fail_validate: bool,
    pub config_response: Option<DotfConfig>,
}

impl MockRepository {
    pub fn new() -> Self {
        Self {
            expectations: Arc::new(Mutex::new(MockRepositoryExpectations::default())),
        }
    }

    pub fn expect_validate_remote(&self, url: &str) -> &Self {
        self.expectations.lock().unwrap().validate_calls.push(url.to_string());
        self
    }

    pub fn fail_validate(&self) -> &Self {
        self.expectations.lock().unwrap().should_fail_validate = true;
        self
    }
}

#[async_trait]
impl Repository for MockRepository {
    async fn validate_remote(&self, url: &str) -> DotfResult<()> {
        let mut exp = self.expectations.lock().unwrap();
        exp.validate_calls.push(url.to_string());

        if exp.should_fail_validate {
            return Err(DotfError::Repository("Mock validation failure".to_string()));
        }

        Ok(())
    }

    // 他のメソッドのモック実装...
}

#[derive(Clone)]
pub struct MockFileSystem {
    pub files: Arc<Mutex<HashMap<String, String>>>,
    pub directories: Arc<Mutex<Vec<String>>>,
    pub symlinks: Arc<Mutex<Vec<(String, String)>>>,
}

impl MockFileSystem {
    pub fn new() -> Self {
        Self {
            files: Arc::new(Mutex::new(HashMap::new())),
            directories: Arc::new(Mutex::new(Vec::new())),
            symlinks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_file(&self, path: &str, content: &str) {
        self.files.lock().unwrap().insert(path.to_string(), content.to_string());
    }
}

#[async_trait]
impl FileSystem for MockFileSystem {
    async fn exists(&self, path: &str) -> DotfResult<bool> {
        Ok(self.files.lock().unwrap().contains_key(path))
    }

    async fn create_symlink(&self, source: &str, target: &str) -> DotfResult<()> {
        self.symlinks.lock().unwrap().push((source.to_string(), target.to_string()));
        Ok(())
    }

    // 他のメソッドのモック実装...
}
```

### Unit Test Example

#### `tests/integration/init_tests.rs`
```rust
use dotf::services::InitService;
use dotf::error::DotfError;
use crate::common::mocks::{MockRepository, MockFileSystem, MockPrompt};

#[tokio::test]
async fn test_init_with_valid_repo_url() {
    // Arrange
    let mock_repo = MockRepository::new();
    let mock_fs = MockFileSystem::new();
    let mock_prompt = MockPrompt::new();

    let service = InitService::new(mock_repo.clone(), mock_fs.clone(), mock_prompt);
    let repo_url = "https://github.com/user/dotfiles.git";

    // Act
    let result = service.init(Some(repo_url.to_string())).await;

    // Assert
    assert!(result.is_ok());

    // Verify interactions
    let expectations = mock_repo.expectations.lock().unwrap();
    assert_eq!(expectations.validate_calls.len(), 1);
    assert_eq!(expectations.validate_calls[0], repo_url);
}

#[tokio::test]
async fn test_init_with_invalid_repo_url() {
    // Arrange
    let mock_repo = MockRepository::new();
    mock_repo.fail_validate();

    let mock_fs = MockFileSystem::new();
    let mock_prompt = MockPrompt::new();

    let service = InitService::new(mock_repo, mock_fs, mock_prompt);

    // Act
    let result = service.init(Some("invalid-url".to_string())).await;

    // Assert
    assert!(matches!(result, Err(DotfError::Repository(_))));
}

#[tokio::test]
async fn test_init_with_interactive_prompt() {
    // Arrange
    let mock_repo = MockRepository::new();
    let mock_fs = MockFileSystem::new();
    let mock_prompt = MockPrompt::new();
    mock_prompt.set_repository_url_response("https://github.com/user/dotfiles.git");

    let service = InitService::new(mock_repo, mock_fs, mock_prompt.clone());

    // Act
    let result = service.init(None).await;

    // Assert
    assert!(result.is_ok());
    assert!(mock_prompt.was_repository_url_asked());
}
```

## 依存関係 (Cargo.toml)

```toml
[package]
name = "dotf"
version = "0.1.0"
edition = "2021"

[dependencies]
# CLI
clap = { version = "4.0", features = ["derive"] }

# Async runtime
tokio = { version = "1.0", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
serde_json = "1.0"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# File system
dirs = "5.0"
tempfile = "3.0"

# Git operations
git2 = "0.18"

# Async traits
async-trait = "0.1"

# Terminal UI
dialoguer = "0.11"
console = "0.15"
indicatif = "0.17"

[dev-dependencies]
# Testing
tokio-test = "0.4"
tempfile = "3.0"
assert_matches = "1.5"

[features]
default = []
```

## 設計原則

1. **Dependency Injection**: トレイトを使用した依存性の注入
2. **Test-Driven Development**: モックを活用したユニットテスト
3. **Error Handling**: `thiserror`を使用した型安全なエラー処理
4. **Async/Await**: 非同期処理による高いパフォーマンス
5. **Modularity**: 機能別の明確なモジュール分割
6. **SOLID Principles**: 単一責任の原則とインターフェース分離
