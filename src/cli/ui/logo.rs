//! ASCII art logo and branding for Dott

use crate::cli::ui::Theme;
use std::time::Duration;
use tokio::time::sleep;

/// ASCII art logo for Dott
pub struct Logo {
    theme: Theme,
}

impl Default for Logo {
    fn default() -> Self {
        Self::new()
    }
}

impl Logo {
    /// Create a new logo instance
    pub fn new() -> Self {
        Self {
            theme: Theme::new(),
        }
    }

    /// Get the main Dott logo
    pub fn main_logo(&self) -> String {
        let logo = r#"
    ██████╗  ██████╗ ████████╗████████╗
    ██╔══██╗██╔═══██╗╚══██╔══╝╚══██╔══╝
    ██║  ██║██║   ██║   ██║      ██║   
    ██║  ██║██║   ██║   ██║      ██║   
    ██████╔╝╚██████╔╝   ██║      ██║   
    ╚═════╝  ╚═════╝    ╚═╝      ╚═╝   
"#;
        self.theme.primary(logo)
    }

    /// Get the compact Dott logo
    pub fn compact_logo(&self) -> String {
        let logo = r#"
   ██████╗  ██████╗ ████████╗████████╗
   ██╔══██╗██╔═══██╗╚══██╔══╝╚══██╔══╝
   ██║  ██║██║   ██║   ██║      ██║   
   ██████╔╝╚██████╔╝   ██║      ██║   
   ╚═════╝  ╚═════╝    ╚═╝      ╚═╝   
"#;
        self.theme.primary(logo)
    }

    /// Get a stylized mini logo
    pub fn mini_logo(&self) -> String {
        format!("{}ott", self.theme.accent("D"))
    }

    /// Get an animated dots pattern
    pub fn dots_pattern(&self) -> String {
        self.theme.muted("● ● ● ● ●")
    }

    /// Create a welcome banner with logo and tagline
    pub fn welcome_banner(&self, version: &str) -> String {
        format!(
            "{}\n{}\n{}\n{}\n",
            self.compact_logo(),
            self.theme.muted(&format!(
                "{}   Modern Dotfile Management Tool",
                " ".repeat(8)
            )),
            self.theme
                .muted(&format!("{}   Version {}", " ".repeat(8), version)),
            self.theme
                .accent(&format!("{}   {}", " ".repeat(8), self.dots_pattern()))
        )
    }

    /// Create a simple branded header
    pub fn header(&self) -> String {
        format!(
            "{} {}",
            self.mini_logo(),
            self.theme.muted("• Modern Dotfile Management")
        )
    }

    /// Animated logo reveal
    pub async fn animated_reveal(&self) -> String {
        // This would be used for more complex animations
        // For now, return the standard logo
        self.compact_logo()
    }
}

/// Installation animation stages
#[derive(Debug, Clone)]
pub enum InstallStage {
    Welcome,
    ValidatingRepository,
    SelectingBranch,
    FetchingConfiguration,
    SettingUpDirectories,
    CloningRepository,
    CreatingSymlinks,
    FinalizeSetup,
    Complete,
}

impl InstallStage {
    /// Get the display message for this stage
    pub fn message(&self) -> &'static str {
        match self {
            InstallStage::Welcome => "Setting up dott",
            InstallStage::ValidatingRepository => "Validating repository URL",
            InstallStage::SelectingBranch => "Selecting branch",
            InstallStage::FetchingConfiguration => "Fetching configuration from repository",
            InstallStage::SettingUpDirectories => "Setting up dott directories",
            InstallStage::CloningRepository => "Cloning dotfiles repository",
            InstallStage::CreatingSymlinks => "Creating symbolic links",
            InstallStage::FinalizeSetup => "Finalizing setup",
            InstallStage::Complete => "Setup complete!",
        }
    }

    /// Get the icon for this stage
    pub fn icon(&self) -> &'static str {
        match self {
            InstallStage::Welcome => "🚀",
            InstallStage::ValidatingRepository => "🔍",
            InstallStage::SelectingBranch => "🌿",
            InstallStage::FetchingConfiguration => "📥",
            InstallStage::SettingUpDirectories => "📁",
            InstallStage::CloningRepository => "📦",
            InstallStage::CreatingSymlinks => "🔗",
            InstallStage::FinalizeSetup => "⚙️",
            InstallStage::Complete => "✨",
        }
    }

    /// Get all stages in order
    pub fn all_stages() -> Vec<InstallStage> {
        vec![
            InstallStage::Welcome,
            InstallStage::ValidatingRepository,
            InstallStage::SelectingBranch,
            InstallStage::FetchingConfiguration,
            InstallStage::SettingUpDirectories,
            InstallStage::CloningRepository,
            InstallStage::CreatingSymlinks,
            InstallStage::FinalizeSetup,
            InstallStage::Complete,
        ]
    }
}

/// Animated installation progress display
pub struct InstallAnimation {
    theme: Theme,
    logo: Logo,
}

impl Default for InstallAnimation {
    fn default() -> Self {
        Self::new()
    }
}

impl InstallAnimation {
    /// Create a new installation animation
    pub fn new() -> Self {
        Self {
            theme: Theme::new(),
            logo: Logo::new(),
        }
    }

    /// Show the welcome screen with logo
    pub async fn show_welcome(&self, version: &str) {
        println!("{}", self.logo.welcome_banner(version));
        self.typewriter_effect("Initializing dott configuration...", 30)
            .await;
        sleep(Duration::from_millis(500)).await;
    }

    /// Show a stage with animation
    pub async fn show_stage(&self, stage: &InstallStage) {
        let stage_text = format!("{} {}", stage.icon(), self.theme.primary(stage.message()));

        println!("\n{}", stage_text);

        // Add loading animation only for stages that actually process something
        match stage {
            InstallStage::SelectingBranch => {
                // No loading animation for branch selection (user input)
            }
            _ => {
                // Add a brief loading animation for other stages
                self.loading_dots(3).await;
            }
        }
    }

    /// Show completion message
    pub async fn show_completion(&self, repo_url: &str) {
        println!("\n{}", "=".repeat(60));
        println!("{}", self.theme.success("🎉 Setup Complete! 🎉"));
        println!("{}", "=".repeat(60));

        println!("\n{}", self.theme.accent("Repository:"));
        println!("  {}", self.theme.value(repo_url));

        println!("\n{}", self.theme.accent("What's next?"));
        println!(
            "  {} Run 'dott status' to see your setup",
            self.theme.primary("→")
        );
        println!(
            "  {} Run 'dott install config' to create symlinks",
            self.theme.primary("→")
        );
        println!(
            "  {} Run 'dott sync' to sync with remote",
            self.theme.primary("→")
        );

        println!("\n{}", self.theme.muted("Happy dotfile management! ✨"));
    }

    /// Typewriter effect for text
    async fn typewriter_effect(&self, text: &str, delay_ms: u64) {
        for char in text.chars() {
            print!("{}", char);
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
            sleep(Duration::from_millis(delay_ms)).await;
        }
        println!();
    }

    /// Loading dots animation
    async fn loading_dots(&self, count: usize) {
        for _ in 0..count {
            print!(".");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
            sleep(Duration::from_millis(300)).await;
        }
        println!();
    }

    /// Progress bar for a stage
    pub fn progress_bar(&self, current: usize, total: usize) -> String {
        let width = 30;
        let filled = (current * width) / total;
        let empty = width - filled;

        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

        format!(
            "[{}] {}/{} {}",
            self.theme.primary(&bar),
            self.theme.accent(&current.to_string()),
            self.theme.muted(&total.to_string()),
            self.theme.muted(&format!("({}%)", (current * 100) / total))
        )
    }
}

/// Celebration effects for successful completion
pub struct CelebrationEffects {
    theme: Theme,
}

impl Default for CelebrationEffects {
    fn default() -> Self {
        Self::new()
    }
}

impl CelebrationEffects {
    /// Create new celebration effects
    pub fn new() -> Self {
        Self {
            theme: Theme::new(),
        }
    }

    /// Show sparkle effect
    pub async fn sparkles(&self) {
        let sparkles = ["✨", "🌟", "⭐", "💫", "🎇"];
        for _ in 0..5 {
            for sparkle in &sparkles {
                print!("{} ", sparkle);
                use std::io::{self, Write};
                io::stdout().flush().unwrap();
                sleep(Duration::from_millis(100)).await;
            }
            print!("\r{}", " ".repeat(15));
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
            sleep(Duration::from_millis(100)).await;
        }
        println!();
    }

    /// Show success banner
    pub fn success_banner(&self) -> String {
        format!(
            "\n{}\n{}\n{}\n",
            self.theme.success("██████████████████████████████"),
            self.theme.success("█  🎉 SETUP COMPLETE! 🎉  █"),
            self.theme.success("██████████████████████████████")
        )
    }
}
