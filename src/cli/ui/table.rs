//! Beautiful table formatting for CLI output

use crate::cli::ui::{Icons, Theme};
use std::fmt;

/// A row in a table
#[derive(Debug, Clone)]
pub struct TableRow {
    pub cells: Vec<String>,
}

impl TableRow {
    /// Create a new table row
    pub fn new(cells: Vec<String>) -> Self {
        Self { cells }
    }

    /// Create a row from string slice
    pub fn from_strings(cells: &[&str]) -> Self {
        Self {
            cells: cells.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// A beautiful table for displaying structured data
#[derive(Debug)]
pub struct Table {
    headers: Vec<String>,
    rows: Vec<TableRow>,
    theme: Theme,
    show_borders: bool,
    show_header: bool,
}

impl Table {
    /// Create a new table
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            rows: Vec::new(),
            theme: Theme::new(),
            show_borders: true,
            show_header: true,
        }
    }

    /// Set table headers
    pub fn headers(mut self, headers: Vec<String>) -> Self {
        self.headers = headers;
        self
    }

    /// Set headers from string slice
    pub fn headers_from_strings(mut self, headers: &[&str]) -> Self {
        self.headers = headers.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add a row to the table
    pub fn add_row(mut self, row: TableRow) -> Self {
        self.rows.push(row);
        self
    }

    /// Add a row from string slice
    pub fn add_row_from_strings(mut self, cells: &[&str]) -> Self {
        self.rows.push(TableRow::from_strings(cells));
        self
    }

    /// Hide borders
    pub fn no_borders(mut self) -> Self {
        self.show_borders = false;
        self
    }

    /// Hide header
    pub fn no_header(mut self) -> Self {
        self.show_header = false;
        self
    }

    /// Calculate column widths
    fn calculate_column_widths(&self) -> Vec<usize> {
        let mut widths = Vec::new();

        // Initialize with header widths
        if self.show_header {
            for header in &self.headers {
                widths.push(display_width(header));
            }
        }

        // Update with row cell widths
        for row in &self.rows {
            for (i, cell) in row.cells.iter().enumerate() {
                if i >= widths.len() {
                    widths.push(0);
                }
                // Calculate display width accounting for emojis and ANSI codes
                let cell_width = display_width(cell);
                widths[i] = widths[i].max(cell_width);
            }
        }

        // Ensure minimum width for readability
        for width in &mut widths {
            *width = (*width).max(3);
        }

        widths
    }

    /// Format a horizontal border
    fn format_border(
        &self,
        widths: &[usize],
        start: &str,
        middle: &str,
        end: &str,
        fill: &str,
    ) -> String {
        if !self.show_borders {
            return String::new();
        }

        let mut border = String::new();
        border.push_str(&self.theme.muted(start));

        for (i, width) in widths.iter().enumerate() {
            if i > 0 {
                border.push_str(&self.theme.muted(middle));
            }
            border.push_str(&self.theme.muted(&fill.repeat(width + 2)));
        }

        border.push_str(&self.theme.muted(end));
        border
    }

    /// Format a table row
    fn format_row(&self, cells: &[String], widths: &[usize], is_header: bool) -> String {
        let mut row = String::new();

        if self.show_borders {
            row.push_str(&self.theme.muted("│"));
        }

        // Process each column, even if no cell exists for it
        let max_cols = widths.len();
        #[allow(clippy::needless_range_loop)]
        for i in 0..max_cols {
            if i > 0 && self.show_borders {
                row.push_str(&self.theme.muted("│"));
            }

            let cell = cells.get(i).map(|c| c.as_str()).unwrap_or("");
            let width = widths[i];
            let content_length = display_width(cell);
            let padding = width.saturating_sub(content_length);

            if self.show_borders {
                row.push(' ');
            }

            // Add cell content
            if is_header && !cell.is_empty() {
                row.push_str(&self.theme.header(cell));
            } else {
                row.push_str(cell);
            }

            // Add padding to reach column width
            row.push_str(&" ".repeat(padding));

            if self.show_borders {
                row.push(' ');
            }
        }

        if self.show_borders {
            row.push_str(&self.theme.muted("│"));
        }

        row
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let widths = self.calculate_column_widths();

        // Top border
        if self.show_borders {
            writeln!(f, "{}", self.format_border(&widths, "┌", "┬", "┐", "─"))?;
        }

        // Header
        if self.show_header && !self.headers.is_empty() {
            writeln!(f, "{}", self.format_row(&self.headers, &widths, true))?;

            if self.show_borders {
                writeln!(f, "{}", self.format_border(&widths, "├", "┼", "┤", "─"))?;
            }
        }

        // Rows
        for (i, row) in self.rows.iter().enumerate() {
            writeln!(f, "{}", self.format_row(&row.cells, &widths, false))?;

            // Separator between rows (optional)
            if self.show_borders && i < self.rows.len() - 1 {
                // Uncomment to add separators between all rows
                // writeln!(f, "{}", self.format_border(&widths, "├", "┼", "┤", "─"))?;
            }
        }

        // Bottom border
        if self.show_borders {
            write!(f, "{}", self.format_border(&widths, "└", "┴", "┘", "─"))?;
        }

        Ok(())
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple list formatter for vertical data
pub struct List {
    items: Vec<String>,
    theme: Theme,
    bullet: String,
    indent: usize,
}

impl List {
    /// Create a new list
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            theme: Theme::new(),
            bullet: Icons::BULLET.to_string(),
            indent: 2,
        }
    }

    /// Add an item to the list
    pub fn add_item(mut self, item: String) -> Self {
        self.items.push(item);
        self
    }

    /// Add an item from string
    pub fn add_item_str(mut self, item: &str) -> Self {
        self.items.push(item.to_string());
        self
    }

    /// Set custom bullet point
    pub fn bullet(mut self, bullet: &str) -> Self {
        self.bullet = bullet.to_string();
        self
    }

    /// Set indent level
    pub fn indent(mut self, indent: usize) -> Self {
        self.indent = indent;
        self
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in &self.items {
            writeln!(
                f,
                "{}{} {}",
                " ".repeat(self.indent),
                self.theme.muted(&self.bullet),
                item
            )?;
        }
        Ok(())
    }
}

impl Default for List {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to strip ANSI color codes for length calculation
fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // Check if this is the start of an ANSI escape sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['

                // Skip the entire escape sequence
                // ANSI escape sequences end with a letter (A-Za-z)
                for next_ch in chars.by_ref() {
                    if next_ch.is_ascii_alphabetic() {
                        break;
                    }
                    // Also handle sequences that end with '~' (like cursor positioning)
                    if next_ch == '~' {
                        break;
                    }
                }
            } else {
                // Not an ANSI sequence, keep the character
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Calculate the display width of a string, accounting for emoji and wide characters
fn display_width(s: &str) -> usize {
    let clean = strip_ansi_codes(s);
    let mut width = 0;
    let mut chars = clean.chars();

    while let Some(ch) = chars.next() {
        let ch_str = ch.to_string();

        // Check for common emoji sequences that take 2 terminal columns
        if matches!(ch_str.as_str(), "✅" | "❌" | "💥" | "🎯" | "🔄") {
            width += 2;
        } else if ch == '⚠' {
            // Check if followed by variation selector
            if chars.as_str().starts_with('\u{fe0f}') {
                chars.next(); // consume variation selector
                width += 2;
            } else {
                width += 1;
            }
        } else if ch.is_ascii() {
            width += 1;
        } else {
            // Non-ASCII characters - for better support, we'd use unicode-width crate
            // For now, assume most take 1 column
            width += 1;
        }
    }

    width
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_stripping() {
        assert_eq!(strip_ansi_codes("hello"), "hello");
        assert_eq!(strip_ansi_codes("\x1b[31mred\x1b[0m"), "red");
        assert_eq!(
            strip_ansi_codes("\x1b[1;32mbold green\x1b[0m"),
            "bold green"
        );
        assert_eq!(strip_ansi_codes("✅ Valid"), "✅ Valid");
    }
}
