//! Color theme and styling definitions for consistent UI

use colored::{Color, Colorize};

/// UI theme with consistent colors and styles
#[derive(Clone, Debug)]
pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    pub muted: Color,
    pub accent: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Blue,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Magenta,
            muted: Color::BrightBlack,
            accent: Color::White,
        }
    }
}

impl Theme {
    /// Create a new theme instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Style text with primary color
    pub fn primary(&self, text: &str) -> String {
        text.color(self.primary).to_string()
    }

    /// Style text with secondary color
    pub fn secondary(&self, text: &str) -> String {
        text.color(self.secondary).to_string()
    }

    /// Style text with success color
    pub fn success(&self, text: &str) -> String {
        text.color(self.success).bold().to_string()
    }

    /// Style text with warning color
    pub fn warning(&self, text: &str) -> String {
        text.color(self.warning).bold().to_string()
    }

    /// Style text with error color
    pub fn error(&self, text: &str) -> String {
        text.color(self.error).bold().to_string()
    }

    /// Style text with info color
    pub fn info(&self, text: &str) -> String {
        text.color(self.info).to_string()
    }

    /// Style text with muted color
    pub fn muted(&self, text: &str) -> String {
        text.color(self.muted).to_string()
    }

    /// Style text with accent color
    pub fn accent(&self, text: &str) -> String {
        text.color(self.accent).bold().to_string()
    }

    /// Style text as a header
    pub fn header(&self, text: &str) -> String {
        text.color(self.primary).bold().underline().to_string()
    }

    /// Style text as a subheader
    pub fn subheader(&self, text: &str) -> String {
        text.color(self.secondary).bold().to_string()
    }

    /// Style text as a label
    pub fn label(&self, text: &str) -> String {
        text.color(self.accent).to_string()
    }

    /// Style text as a value
    pub fn value(&self, text: &str) -> String {
        text.color(self.primary).to_string()
    }

    /// Style text as a path
    pub fn path(&self, text: &str) -> String {
        text.color(self.info).italic().to_string()
    }

    /// Style text as a command
    pub fn command(&self, text: &str) -> String {
        text.color(self.accent).on_color(Color::BrightBlack).to_string()
    }
}