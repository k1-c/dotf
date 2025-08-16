//! Text formatting utilities for beautiful CLI output

use crate::cli::ui::{Icons, Theme};
use std::fmt;

/// A beautiful message formatter with consistent styling
pub struct MessageFormatter {
    theme: Theme,
}

impl Default for MessageFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageFormatter {
    /// Create a new message formatter
    pub fn new() -> Self {
        Self {
            theme: Theme::new(),
        }
    }

    /// Format a success message
    pub fn success(&self, message: &str) -> String {
        format!("{} {}", Icons::SUCCESS, self.theme.success(message))
    }

    /// Format an error message
    pub fn error(&self, message: &str) -> String {
        format!("{} {}", Icons::ERROR, self.theme.error(message))
    }

    /// Format a warning message
    pub fn warning(&self, message: &str) -> String {
        format!("{} {}", Icons::WARNING, self.theme.warning(message))
    }

    /// Format an info message
    pub fn info(&self, message: &str) -> String {
        format!("{} {}", Icons::INFO, self.theme.info(message))
    }

    /// Format a question
    pub fn question(&self, message: &str) -> String {
        format!("{} {}", Icons::QUESTION, self.theme.accent(message))
    }

    /// Format a header with decorative borders
    pub fn header(&self, title: &str) -> String {
        let border = "═".repeat(title.len() + 4);
        format!(
            "{}\n  {}  \n{}",
            self.theme.primary(&border),
            self.theme.header(title),
            self.theme.primary(&border)
        )
    }

    /// Format a section header
    pub fn section(&self, title: &str) -> String {
        format!(
            "\n{} {}\n{}",
            Icons::ARROW_RIGHT,
            self.theme.subheader(title),
            self.theme.muted(&"─".repeat(title.len() + 2))
        )
    }

    /// Format a key-value pair
    pub fn key_value(&self, key: &str, value: &str) -> String {
        format!("{}: {}", self.theme.label(key), self.theme.value(value))
    }

    /// Format a path
    pub fn path(&self, path: &str) -> String {
        self.theme.path(path)
    }

    /// Format a command
    pub fn command(&self, cmd: &str) -> String {
        self.theme.command(&format!(" {} ", cmd))
    }

    /// Format an operation status
    pub fn status(&self, operation: &str, status: OperationStatus) -> String {
        let (icon, styled_status) = match status {
            OperationStatus::Success => (Icons::SUCCESS, self.theme.success("SUCCESS")),
            OperationStatus::Failed => (Icons::ERROR, self.theme.error("FAILED")),
            OperationStatus::Warning => (Icons::WARNING, self.theme.warning("WARNING")),
            OperationStatus::InProgress => (Icons::SYNC, self.theme.info("IN PROGRESS")),
            OperationStatus::Skipped => (Icons::ARROW_RIGHT, self.theme.muted("SKIPPED")),
        };

        format!("{} {} {}", icon, self.theme.label(operation), styled_status)
    }

    /// Format a progress message
    pub fn progress(&self, current: usize, total: usize, message: &str) -> String {
        format!(
            "{} [{}/{}] {}",
            Icons::SYNC,
            self.theme.accent(&current.to_string()),
            self.theme.muted(&total.to_string()),
            self.theme.primary(message)
        )
    }

    /// Format a file operation
    pub fn file_operation(&self, operation: &str, from: &str, to: &str) -> String {
        format!(
            "{} {} {} {} {}",
            Icons::FILE,
            self.theme.label(operation),
            self.theme.path(from),
            Icons::ARROW_RIGHT,
            self.theme.path(to)
        )
    }

    /// Format a git operation
    pub fn git_operation(&self, operation: &str, details: &str) -> String {
        format!(
            "{} {} {}",
            Icons::GIT,
            self.theme.label(operation),
            self.theme.value(details)
        )
    }

    /// Format an indented message
    pub fn indent(&self, message: &str, level: usize) -> String {
        let indent = "  ".repeat(level);
        format!("{}{}", indent, message)
    }

    /// Format a tree-like structure
    pub fn tree_item(&self, message: &str, is_last: bool, level: usize) -> String {
        let prefix = if level == 0 {
            String::new()
        } else {
            let mut prefix = "  ".repeat(level - 1);
            if is_last {
                prefix.push_str(Icons::TREE_LAST);
            } else {
                prefix.push_str(Icons::TREE_BRANCH);
            }
            prefix.push(' ');
            self.theme.muted(&prefix)
        };

        format!("{}{}", prefix, message)
    }

    /// Format a summary box
    pub fn summary_box(&self, title: &str, items: &[(&str, &str)]) -> String {
        let mut result = String::new();

        // Title
        result.push_str(&format!("{}\n", self.section(title)));

        // Items
        for (key, value) in items {
            result.push_str(&format!("  {}\n", self.key_value(key, value)));
        }

        result
    }
}

/// Operation status for formatting
#[derive(Debug, Clone, Copy)]
pub enum OperationStatus {
    Success,
    Failed,
    Warning,
    InProgress,
    Skipped,
}

/// A formatted output builder for complex layouts
pub struct OutputBuilder {
    content: Vec<String>,
    formatter: MessageFormatter,
}

impl OutputBuilder {
    /// Create a new output builder
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
            formatter: MessageFormatter::new(),
        }
    }

    /// Add a line to the output
    pub fn line(mut self, text: &str) -> Self {
        self.content.push(text.to_string());
        self
    }

    /// Add an empty line
    pub fn empty_line(mut self) -> Self {
        self.content.push(String::new());
        self
    }

    /// Add a success message
    pub fn success(mut self, message: &str) -> Self {
        self.content.push(self.formatter.success(message));
        self
    }

    /// Add an error message
    pub fn error(mut self, message: &str) -> Self {
        self.content.push(self.formatter.error(message));
        self
    }

    /// Add a warning message
    pub fn warning(mut self, message: &str) -> Self {
        self.content.push(self.formatter.warning(message));
        self
    }

    /// Add an info message
    pub fn info(mut self, message: &str) -> Self {
        self.content.push(self.formatter.info(message));
        self
    }

    /// Add a section header
    pub fn section(mut self, title: &str) -> Self {
        self.content.push(self.formatter.section(title));
        self
    }

    /// Add a key-value pair
    pub fn key_value(mut self, key: &str, value: &str) -> Self {
        self.content
            .push(format!("  {}", self.formatter.key_value(key, value)));
        self
    }

    /// Add an indented line
    pub fn indent(mut self, message: &str, level: usize) -> Self {
        self.content.push(self.formatter.indent(message, level));
        self
    }

    /// Build the final output
    pub fn build(self) -> String {
        self.content.join("\n")
    }

    /// Print the output to stdout
    pub fn print(self) {
        println!("{}", self.build());
    }
}

impl Default for OutputBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for OutputBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content.join("\n"))
    }
}
