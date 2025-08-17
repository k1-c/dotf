//! High-level UI components combining multiple UI elements

use crate::cli::ui::{Icons, MessageFormatter, OperationStatus, Theme};
use crate::core::symlinks::SymlinkStatus;

/// High-level UI components for common CLI patterns
pub struct UiComponents {
    formatter: MessageFormatter,
    theme: Theme,
}

impl Default for UiComponents {
    fn default() -> Self {
        Self::new()
    }
}

impl UiComponents {
    /// Create new UI components
    pub fn new() -> Self {
        Self {
            formatter: MessageFormatter::new(),
            theme: Theme::new(),
        }
    }

    /// Display a welcome banner
    pub fn welcome_banner(&self, version: &str) -> String {
        format!(
            "{}\n{}\n{}\n{}\n",
            self.theme
                .primary("╔══════════════════════════════════════╗"),
            self.theme
                .primary(&format!("║  {}  {} {} ║", Icons::ROCKET, "Dotf", version)),
            self.theme
                .primary("║      Modern Dotfile Management      ║"),
            self.theme
                .primary("╚══════════════════════════════════════╝"),
        )
    }

    /// Display repository status summary
    pub fn repository_status(
        &self,
        is_clean: bool,
        behind: usize,
        ahead: usize,
        branch: &str,
    ) -> String {
        let mut output = Vec::new();

        output.push(self.formatter.section("Repository Status"));
        output.push(self.formatter.key_value("Branch", branch));

        if is_clean {
            output.push(format!(
                "  {}",
                self.formatter.success("Working tree is clean")
            ));
        } else {
            output.push(format!(
                "  {}",
                self.formatter
                    .warning("Working tree has uncommitted changes")
            ));
        }

        if behind > 0 {
            output.push(format!("  {} {} commits behind", Icons::DOWNLOAD, behind));
        }

        if ahead > 0 {
            output.push(format!("  {} {} commits ahead", Icons::UPLOAD, ahead));
        }

        if behind == 0 && ahead == 0 {
            output.push(format!(
                "  {}",
                self.formatter.success("Up to date with remote")
            ));
        }

        output.join("\n")
    }

    /// Display symlink status summary with a beautiful list
    pub fn symlinks_status_table(&self, symlinks: &[SymlinkDetail], repo_path: &str) -> String {
        if symlinks.is_empty() {
            return self.formatter.info("No symlinks configured");
        }

        let mut output = Vec::new();
        output.push(self.formatter.section("Symlinks Status"));

        // Group symlinks by status for better organization
        let mut by_status: std::collections::HashMap<String, Vec<&SymlinkDetail>> =
            std::collections::HashMap::new();

        for symlink in symlinks {
            let status_key = format!("{:?}", symlink.status);
            by_status.entry(status_key).or_default().push(symlink);
        }

        // Display order: Conflicts first, then Invalid, then others
        let status_order = [
            "Conflict",
            "InvalidTarget",
            "Missing",
            "Broken",
            "Modified",
            "Valid",
        ];

        for status_name in &status_order {
            if let Some(links) = by_status.get(*status_name) {
                // Sort links alphabetically by source path within each group
                let mut sorted_links = links.clone();
                sorted_links.sort_by(|a, b| a.source_path.cmp(&b.source_path));

                for symlink in sorted_links {
                    let (status_icon, status_text) = match symlink.status {
                        SymlinkStatus::Valid => (Icons::VALID, self.theme.success("Valid")),
                        SymlinkStatus::Missing => (Icons::MISSING, self.theme.error("Missing")),
                        SymlinkStatus::Broken => (Icons::BROKEN, self.theme.error("Broken")),
                        SymlinkStatus::Conflict => {
                            (Icons::CONFLICT, self.theme.warning("Conflict"))
                        }
                        SymlinkStatus::InvalidTarget => {
                            (Icons::INVALID_TARGET, self.theme.warning("Wrong target"))
                        }
                        SymlinkStatus::Modified => (Icons::MODIFIED, self.theme.info("Modified")),
                    };

                    // Convert home directory to ~ notation for target display
                    let home_dir = dirs::home_dir().map(|d| d.to_string_lossy().to_string());
                    let target_display = if let Some(ref home) = home_dir {
                        symlink.target_path.replace(home, "~")
                    } else {
                        symlink.target_path.clone()
                    };

                    // For source, remove the repository path prefix
                    let source_display = if symlink.source_path.starts_with(repo_path) {
                        let stripped = symlink
                            .source_path
                            .strip_prefix(repo_path)
                            .unwrap_or(&symlink.source_path);
                        if let Some(without_slash) = stripped.strip_prefix('/') {
                            without_slash.to_string()
                        } else {
                            stripped.to_string()
                        }
                    } else if let Some(ref home) = home_dir {
                        symlink.source_path.replace(home, "~")
                    } else {
                        symlink.source_path.clone()
                    };

                    // Format the entry
                    let status_part = format!("{} {}", status_icon, status_text);
                    let path_part = format!(
                        "{} → {}",
                        self.theme.path(&source_display),
                        self.theme.path(&target_display)
                    );

                    // Add details if necessary
                    let details = match symlink.status {
                        SymlinkStatus::InvalidTarget => Some(self.theme.muted(" (wrong target)")),
                        SymlinkStatus::Missing => Some(self.theme.muted(" (not created)")),
                        SymlinkStatus::Broken => Some(self.theme.muted(" (target missing)")),
                        SymlinkStatus::Conflict => Some(self.theme.muted(" (file exists)")),
                        SymlinkStatus::Modified => Some(self.theme.muted(" (content changed)")),
                        SymlinkStatus::Valid => None,
                    };

                    // Display on a single line
                    if let Some(detail) = details {
                        output.push(format!("  {} {}{}", status_part, path_part, detail));
                    } else {
                        output.push(format!("  {} {}", status_part, path_part));
                    }
                }
            }
        }

        let result = output.join("\n");
        format!("{}\n", result)
    }

    /// Display symlink status summary (compact version)
    #[allow(clippy::too_many_arguments)]
    pub fn symlinks_status_summary(
        &self,
        total: usize,
        valid: usize,
        missing: usize,
        broken: usize,
        conflicts: usize,
        invalid_targets: usize,
        modified: usize,
    ) -> String {
        let total_str = total.to_string();
        let valid_str = format!("{} {}", valid, Icons::SUCCESS);
        let missing_str = format!("{} {}", missing, Icons::ERROR);
        let broken_str = format!("{} {}", broken, Icons::BROKEN);
        let conflicts_str = format!("{} {}", conflicts, Icons::WARNING);
        let invalid_targets_str = format!("{} {}", invalid_targets, Icons::INVALID_TARGET);
        let modified_str = format!("{} {}", modified, Icons::MODIFIED);

        let mut items = Vec::new();

        items.push(("Total", total_str.as_str()));
        items.push(("Valid", valid_str.as_str()));

        if missing > 0 {
            items.push(("Missing", missing_str.as_str()));
        }
        if broken > 0 {
            items.push(("Broken", broken_str.as_str()));
        }
        if conflicts > 0 {
            items.push(("Conflicts", conflicts_str.as_str()));
        }
        if invalid_targets > 0 {
            items.push(("Invalid targets", invalid_targets_str.as_str()));
        }
        if modified > 0 {
            items.push(("Modified", modified_str.as_str()));
        }

        self.formatter.summary_box("Symlinks Summary", &items)
    }

    /// Display configuration summary
    pub fn config_summary(
        &self,
        is_valid: bool,
        symlinks_count: usize,
        scripts_count: usize,
        platforms: &[String],
        errors: &[String],
        warnings: &[String],
    ) -> String {
        let mut output = Vec::new();

        output.push(self.formatter.section("Configuration Summary"));

        if is_valid {
            output.push(format!(
                "  {}",
                self.formatter.success("Configuration is valid")
            ));
        } else {
            output.push(format!(
                "  {}",
                self.formatter.error("Configuration has issues")
            ));
        }

        output.push(format!(
            "  {}",
            self.formatter
                .key_value("Symlinks", &symlinks_count.to_string())
        ));
        output.push(format!(
            "  {}",
            self.formatter
                .key_value("Scripts", &scripts_count.to_string())
        ));

        if !platforms.is_empty() {
            output.push(format!(
                "  {}",
                self.formatter.key_value("Platforms", &platforms.join(", "))
            ));
        }

        if !errors.is_empty() {
            output.push(format!("\n  {} Errors:", Icons::ERROR));
            for error in errors {
                output.push(format!("    {} {}", Icons::BULLET, self.theme.error(error)));
            }
        }

        if !warnings.is_empty() {
            output.push(format!("\n  {} Warnings:", Icons::WARNING));
            for warning in warnings {
                output.push(format!(
                    "    {} {}",
                    Icons::BULLET,
                    self.theme.warning(warning)
                ));
            }
        }

        output.join("\n")
    }

    /// Display backup list
    pub fn backup_list(&self, backups: &[BackupEntry]) -> String {
        if backups.is_empty() {
            return self.formatter.info("No backups found");
        }

        let mut output = Vec::new();
        output.push(self.formatter.section("Available Backups"));

        for backup in backups {
            let original = self.theme.path(&backup.original_path);
            let backup_path = self.theme.muted(&backup.backup_path);
            let created = self.theme.muted(&backup.created_at);

            output.push(format!("  {} {}", original, created));
            output.push(format!("    {}", backup_path));
        }

        let result = output.join("\n");
        format!("{}\n", result)
    }

    /// Display operation results
    pub fn operation_results(&self, title: &str, results: &[OperationResult]) -> String {
        let mut output = Vec::new();

        output.push(self.formatter.section(title));

        for result in results {
            let status_display = self.formatter.status(&result.operation, result.status);
            output.push(format!("  {}", status_display));

            if let Some(ref details) = result.details {
                output.push(format!("    {}", self.theme.muted(details)));
            }
        }

        output.join("\n")
    }

    /// Display a progress summary
    pub fn progress_summary(&self, completed: usize, total: usize, operation: &str) -> String {
        if total == 0 {
            return self.formatter.info(&format!("No {} to process", operation));
        }

        let percentage = (completed as f64 / total as f64 * 100.0) as u8;

        if completed == total {
            self.formatter
                .success(&format!("All {} completed", operation))
        } else {
            self.formatter.info(&format!(
                "{}/{} {} completed ({}%)",
                completed, total, operation, percentage
            ))
        }
    }

    /// Display an error with suggestions
    pub fn error_with_suggestions(&self, error: &str, suggestions: &[&str]) -> String {
        let mut output = Vec::new();

        output.push(self.formatter.error(error));

        if !suggestions.is_empty() {
            output.push(String::new());
            output.push(self.formatter.info("Suggestions:"));
            for suggestion in suggestions {
                output.push(format!("  {} {}", Icons::BULLET, suggestion));
            }
        }

        output.join("\n")
    }
}

/// Symlink detail for display
pub struct SymlinkDetail {
    pub status: SymlinkStatus,
    pub target_path: String,
    pub source_path: String,
    pub current_target: Option<String>,
}

/// Backup entry for display
pub struct BackupEntry {
    pub original_path: String,
    pub backup_path: String,
    pub created_at: String,
}

/// Operation result for display
pub struct OperationResult {
    pub operation: String,
    pub status: OperationStatus,
    pub details: Option<String>,
}
