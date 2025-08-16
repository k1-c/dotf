use crate::core::config::DottConfig;
use crate::error::{DottError, DottResult};
use crate::traits::repository::{Repository, RepositoryStatus};
use async_trait::async_trait;
use std::process::Command;

pub struct GitRepository;

impl Default for GitRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl GitRepository {
    pub fn new() -> Self {
        Self
    }

    fn run_git_command(&self, args: &[&str], cwd: Option<&str>) -> DottResult<String> {
        let mut cmd = Command::new("git");
        cmd.args(args);

        if let Some(cwd) = cwd {
            cmd.current_dir(cwd);
        }

        let output = cmd
            .output()
            .map_err(|e| DottError::Git(format!("Failed to run git command: {}", e)))?;

        if !output.status.success() {
            return Err(DottError::Git(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

#[async_trait]
impl Repository for GitRepository {
    async fn validate_remote(&self, url: &str) -> DottResult<()> {
        // Use git ls-remote to validate the repository
        self.run_git_command(&["ls-remote", "--exit-code", url], None)?;
        Ok(())
    }

    async fn fetch_config(&self, url: &str) -> DottResult<DottConfig> {
        // Create a temporary directory for sparse checkout
        let temp_dir = tempfile::tempdir().map_err(DottError::Io)?;
        let temp_path = temp_dir.path().to_string_lossy();

        // Initialize git repo
        self.run_git_command(&["init"], Some(&temp_path))?;

        // Add remote
        self.run_git_command(&["remote", "add", "origin", url], Some(&temp_path))?;

        // Enable sparse checkout
        self.run_git_command(&["config", "core.sparseCheckout", "true"], Some(&temp_path))?;

        // Configure sparse checkout to only get dott.toml
        let sparse_file = temp_dir.path().join(".git/info/sparse-checkout");
        std::fs::write(&sparse_file, "dott.toml\n.dott/dott.toml").map_err(DottError::Io)?;

        // Get default branch and fetch
        let default_branch = self
            .get_default_branch(url)
            .await
            .unwrap_or_else(|_| "main".to_string());
        self.run_git_command(
            &["fetch", "--depth=1", "origin", &default_branch],
            Some(&temp_path),
        )?;

        // Checkout
        self.run_git_command(&["checkout", &default_branch], Some(&temp_path))?;

        // Read dott.toml
        let config_path = temp_dir.path().join("dott.toml");
        let alt_config_path = temp_dir.path().join(".dott/dott.toml");

        let config_content = if config_path.exists() {
            std::fs::read_to_string(config_path).map_err(DottError::Io)?
        } else if alt_config_path.exists() {
            std::fs::read_to_string(alt_config_path).map_err(DottError::Io)?
        } else {
            return Err(DottError::Config(
                "dott.toml not found in repository".to_string(),
            ));
        };

        toml::from_str(&config_content)
            .map_err(|e| DottError::Config(format!("Invalid dott.toml: {}", e)))
    }

    async fn fetch_config_from_branch(&self, url: &str, branch: &str) -> DottResult<DottConfig> {
        // Create a temporary directory for sparse checkout
        let temp_dir = tempfile::tempdir().map_err(DottError::Io)?;
        let temp_path = temp_dir.path().to_string_lossy();

        // Initialize git repo
        self.run_git_command(&["init"], Some(&temp_path))?;

        // Add remote
        self.run_git_command(&["remote", "add", "origin", url], Some(&temp_path))?;

        // Enable sparse checkout
        self.run_git_command(&["config", "core.sparseCheckout", "true"], Some(&temp_path))?;

        // Configure sparse checkout to only get dott.toml
        let sparse_file = temp_dir.path().join(".git/info/sparse-checkout");
        std::fs::write(&sparse_file, "dott.toml\n.dott/dott.toml").map_err(DottError::Io)?;

        // Fetch the specific branch
        self.run_git_command(&["fetch", "--depth=1", "origin", branch], Some(&temp_path))?;

        // Checkout the branch
        self.run_git_command(&["checkout", branch], Some(&temp_path))?;

        // Read dott.toml
        let config_path = temp_dir.path().join("dott.toml");
        let alt_config_path = temp_dir.path().join(".dott/dott.toml");

        let config_content = if config_path.exists() {
            std::fs::read_to_string(config_path).map_err(DottError::Io)?
        } else if alt_config_path.exists() {
            std::fs::read_to_string(alt_config_path).map_err(DottError::Io)?
        } else {
            return Err(DottError::Config(
                "dott.toml not found in repository".to_string(),
            ));
        };

        toml::from_str(&config_content)
            .map_err(|e| DottError::Config(format!("Invalid dott.toml: {}", e)))
    }

    async fn clone(&self, url: &str, destination: &str) -> DottResult<()> {
        // Get default branch and clone with that branch
        let default_branch = self
            .get_default_branch(url)
            .await
            .unwrap_or_else(|_| "main".to_string());
        self.run_git_command(
            &["clone", "--branch", &default_branch, url, destination],
            None,
        )?;
        Ok(())
    }

    async fn clone_branch(&self, url: &str, branch: &str, destination: &str) -> DottResult<()> {
        self.run_git_command(&["clone", "--branch", branch, url, destination], None)?;
        Ok(())
    }

    async fn pull(&self, repo_path: &str) -> DottResult<()> {
        // Get the current branch
        let current_branch =
            self.run_git_command(&["rev-parse", "--abbrev-ref", "HEAD"], Some(repo_path))?;

        // Pull from origin with the current branch
        self.run_git_command(
            &["pull", "--rebase", "origin", &current_branch],
            Some(repo_path),
        )?;
        Ok(())
    }

    async fn get_status(&self, repo_path: &str) -> DottResult<RepositoryStatus> {
        // Check if working tree is clean
        let status_output = self.run_git_command(&["status", "--porcelain"], Some(repo_path))?;
        let is_clean = status_output.is_empty();

        // Get current branch
        let current_branch =
            self.run_git_command(&["rev-parse", "--abbrev-ref", "HEAD"], Some(repo_path))?;

        // Fetch to get latest remote info
        let _ = self.run_git_command(&["fetch"], Some(repo_path));

        // Get ahead/behind counts
        let rev_list = self
            .run_git_command(
                &["rev-list", "--left-right", "--count", "HEAD...@{u}"],
                Some(repo_path),
            )
            .unwrap_or_else(|_| "0\t0".to_string());

        let parts: Vec<&str> = rev_list.split('\t').collect();
        let ahead_count = parts.first()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        let behind_count = parts
            .get(1)
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);

        Ok(RepositoryStatus {
            is_clean,
            ahead_count,
            behind_count,
            current_branch,
        })
    }

    async fn get_remote_url(&self, repo_path: &str) -> DottResult<String> {
        self.run_git_command(&["config", "--get", "remote.origin.url"], Some(repo_path))
    }

    async fn is_file_modified(&self, repo_path: &str, file_path: &str) -> DottResult<bool> {
        // Check if file has local changes using git status --porcelain
        let output =
            self.run_git_command(&["status", "--porcelain", file_path], Some(repo_path))?;

        // If output is not empty, the file has changes
        // Git status --porcelain format:
        // - First column: index status
        // - Second column: working tree status
        // - If either column is not empty or space, file is modified
        Ok(!output.trim().is_empty())
    }

    async fn get_default_branch(&self, url: &str) -> DottResult<String> {
        // Use git ls-remote to get the default branch (HEAD)
        let output = self.run_git_command(&["ls-remote", "--symref", url, "HEAD"], None)?;

        // Parse output to find the default branch
        // Format: "ref: refs/heads/main\tHEAD"
        for line in output.lines() {
            if line.starts_with("ref: refs/heads/") {
                if let Some(branch) = line.split('\t').next() {
                    if let Some(branch_name) = branch.strip_prefix("ref: refs/heads/") {
                        return Ok(branch_name.to_string());
                    }
                }
            }
        }

        // Fallback to "main" if we can't determine the default branch
        Ok("main".to_string())
    }

    async fn branch_exists(&self, url: &str, branch: &str) -> DottResult<bool> {
        // Use git ls-remote to check if branch exists
        let result = self.run_git_command(&["ls-remote", "--heads", url, branch], None);

        match result {
            Ok(output) => {
                // If output contains lines, the branch exists
                Ok(!output.trim().is_empty())
            }
            Err(_) => {
                // If command fails, assume branch doesn't exist or repo is unreachable
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_repository_creation() {
        let repo = GitRepository::new();
        // Just ensure we can create an instance
        let _ = repo;
    }
}
