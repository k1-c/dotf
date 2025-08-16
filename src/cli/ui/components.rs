//! High-level UI components combining multiple UI elements

use crate::cli::ui::{Icons, MessageFormatter, OperationStatus, Table, Theme};
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
                .primary(&format!("║  {}  {} {} ║", Icons::ROCKET, "Dott", version)),
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

    /// Display symlink status summary with a beautiful table
    pub fn symlinks_status_table(&self, symlinks: &[SymlinkDetail]) -> String {
        if symlinks.is_empty() {
            return self.formatter.info("No symlinks configured");
        }

        let mut table =
            Table::new().headers_from_strings(&["Status", "Target", "Source", "Details"]);

        for symlink in symlinks {
            let (status_icon, status_text) = match symlink.status {
                SymlinkStatus::Valid => (Icons::VALID, self.theme.success("Valid")),
                SymlinkStatus::Missing => (Icons::MISSING, self.theme.error("Missing")),
                SymlinkStatus::Broken => (Icons::BROKEN, self.theme.error("Broken")),
                SymlinkStatus::Conflict => (Icons::CONFLICT, self.theme.warning("Conflict")),
                SymlinkStatus::InvalidTarget => {
                    (Icons::INVALID_TARGET, self.theme.warning("Invalid Target"))
                }
                SymlinkStatus::Modified => (Icons::MODIFIED, self.theme.info("Modified")),
            };

            let status_col = format!("{} {}", status_icon, status_text);
            let target_col = self.theme.path(&symlink.target_path);
            let source_col = self.theme.path(&symlink.source_path);

            let details_col = match symlink.status {
                SymlinkStatus::InvalidTarget => {
                    if let Some(ref current_target) = symlink.current_target {
                        format!("Points to: {}", self.theme.muted(current_target))
                    } else {
                        String::new()
                    }
                }
                SymlinkStatus::Missing => self.theme.muted("Link not created").to_string(),
                SymlinkStatus::Broken => self.theme.muted("Target missing").to_string(),
                SymlinkStatus::Conflict => self.theme.muted("File exists").to_string(),
                SymlinkStatus::Modified => self.theme.muted("Content changed").to_string(),
                SymlinkStatus::Valid => {
                    String::new() // 正常な場合は詳細不要
                }
            };

            table =
                table.add_row_from_strings(&[&status_col, &target_col, &source_col, &details_col]);
        }

        format!("{}\n{}", self.formatter.section("Symlinks Status"), table)
    }

    /// Display symlink status summary (compact version)
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

        let mut table =
            Table::new().headers_from_strings(&["Original Path", "Backup Location", "Created"]);

        for backup in backups {
            table = table.add_row_from_strings(&[
                &self.theme.path(&backup.original_path),
                &self.theme.path(&backup.backup_path),
                &backup.created_at,
            ]);
        }

        format!("{}\n{}", self.formatter.section("Available Backups"), table)
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
        let status = if completed == total {
            self.formatter
                .success(&format!("All {} completed", operation))
        } else {
            self.formatter.info(&format!(
                "{}/{} {} completed ({}%)",
                completed, total, operation, percentage
            ))
        };

        status
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
