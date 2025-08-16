//! Beautiful spinner and progress indicators

use crate::cli::ui::{Icons, Theme};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use std::fmt::Write;
use std::time::Duration;

/// A beautiful spinner for long-running operations
pub struct Spinner {
    bar: ProgressBar,
    theme: Theme,
}

impl Spinner {
    /// Create a new spinner with a message
    pub fn new(message: &str) -> Self {
        let theme = Theme::new();
        let bar = ProgressBar::new_spinner();

        bar.set_style(
            ProgressStyle::with_template(&format!(
                "{} {{spinner:.cyan}} {}",
                Icons::GEAR,
                theme.primary(message)
            ))
            .unwrap()
            .tick_strings(Icons::SPINNER_FRAMES),
        );

        bar.enable_steady_tick(Duration::from_millis(80));

        Self { bar, theme }
    }

    /// Update the spinner message
    pub fn set_message(&self, message: &str) {
        self.bar.set_style(
            ProgressStyle::with_template(&format!(
                "{} {{spinner:.cyan}} {}",
                Icons::GEAR,
                self.theme.primary(message)
            ))
            .unwrap()
            .tick_strings(Icons::SPINNER_FRAMES),
        );
    }

    /// Finish the spinner with a success message
    pub fn finish_with_success(&self, message: &str) {
        self.bar.finish_with_message(format!(
            "{} {}",
            Icons::SUCCESS,
            self.theme.success(message)
        ));
    }

    /// Finish the spinner with an error message
    pub fn finish_with_error(&self, message: &str) {
        self.bar
            .finish_with_message(format!("{} {}", Icons::ERROR, self.theme.error(message)));
    }

    /// Finish the spinner with a warning message
    pub fn finish_with_warning(&self, message: &str) {
        self.bar.finish_with_message(format!(
            "{} {}",
            Icons::WARNING,
            self.theme.warning(message)
        ));
    }

    /// Finish the spinner and clear it
    pub fn finish_and_clear(&self) {
        self.bar.finish_and_clear();
    }
}

/// A progress bar for operations with known progress
pub struct ProgressIndicator {
    bar: ProgressBar,
    theme: Theme,
}

impl ProgressIndicator {
    /// Create a new progress bar
    pub fn new(total: u64, message: &str) -> Self {
        let theme = Theme::new();
        let bar = ProgressBar::new(total);

        bar.set_style(
            ProgressStyle::with_template(&format!(
                "{} [{{elapsed_precise}}] [{{wide_bar:.cyan/blue}}] {{pos}}/{{len}} {{msg}}",
                Icons::SYNC
            ))
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
                write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
            })
            .progress_chars("##-"),
        );

        bar.set_message(theme.primary(message));

        Self { bar, theme }
    }

    /// Increment progress by 1
    pub fn inc(&self) {
        self.bar.inc(1);
    }

    /// Set current position
    pub fn set_position(&self, pos: u64) {
        self.bar.set_position(pos);
    }

    /// Update the message
    pub fn set_message(&self, message: &str) {
        self.bar.set_message(self.theme.primary(message));
    }

    /// Finish with success
    pub fn finish_with_success(&self, message: &str) {
        self.bar.finish_with_message(format!(
            "{} {}",
            Icons::SUCCESS,
            self.theme.success(message)
        ));
    }

    /// Finish with error
    pub fn finish_with_error(&self, message: &str) {
        self.bar
            .finish_with_message(format!("{} {}", Icons::ERROR, self.theme.error(message)));
    }
}

/// Multi-progress manager for handling multiple progress bars
pub struct MultiProgress {
    multi: indicatif::MultiProgress,
    theme: Theme,
}

impl MultiProgress {
    /// Create a new multi-progress manager
    pub fn new() -> Self {
        Self {
            multi: indicatif::MultiProgress::new(),
            theme: Theme::new(),
        }
    }

    /// Add a spinner to the multi-progress
    pub fn add_spinner(&self, message: &str) -> ProgressBar {
        let bar = self.multi.add(ProgressBar::new_spinner());

        bar.set_style(
            ProgressStyle::with_template(&format!(
                "{} {{spinner:.cyan}} {}",
                Icons::GEAR,
                self.theme.primary(message)
            ))
            .unwrap()
            .tick_strings(Icons::SPINNER_FRAMES),
        );

        bar.enable_steady_tick(Duration::from_millis(80));
        bar
    }

    /// Add a progress bar to the multi-progress
    pub fn add_progress(&self, total: u64, message: &str) -> ProgressBar {
        let bar = self.multi.add(ProgressBar::new(total));

        bar.set_style(
            ProgressStyle::with_template(&format!(
                "{} [{{elapsed_precise}}] [{{wide_bar:.cyan/blue}}] {{pos}}/{{len}} {{msg}}",
                Icons::SYNC
            ))
            .unwrap()
            .progress_chars("##-"),
        );

        bar.set_message(self.theme.primary(message));
        bar
    }

    /// Clear all progress bars
    pub fn clear(&self) -> std::io::Result<()> {
        self.multi.clear()
    }
}

impl Default for MultiProgress {
    fn default() -> Self {
        Self::new()
    }
}
