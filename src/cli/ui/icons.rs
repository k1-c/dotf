//! Icon definitions for consistent CLI output

/// Collection of Unicode icons for various states and actions
pub struct Icons;

impl Icons {
    // Status indicators
    pub const SUCCESS: &'static str = "✅";
    pub const ERROR: &'static str = "❌";
    pub const WARNING: &'static str = "⚠️";
    pub const INFO: &'static str = "ℹ️";
    pub const QUESTION: &'static str = "❓";
    pub const CHECKMARK: &'static str = "✓";
    pub const CROSS: &'static str = "✗";

    // Actions
    pub const SYNC: &'static str = "🔄";
    pub const DOWNLOAD: &'static str = "⬇️";
    pub const UPLOAD: &'static str = "⬆️";
    pub const INSTALL: &'static str = "📦";
    pub const LINK: &'static str = "🔗";
    pub const UNLINK: &'static str = "⛓️‍💥";
    pub const BACKUP: &'static str = "💾";
    pub const RESTORE: &'static str = "🔄";
    pub const EDIT: &'static str = "✏️";
    pub const DELETE: &'static str = "🗑️";
    pub const COPY: &'static str = "📋";
    pub const MOVE: &'static str = "🚚";

    // Files and folders
    pub const FILE: &'static str = "📄";
    pub const FOLDER: &'static str = "📁";
    pub const CONFIG: &'static str = "⚙️";
    pub const SCRIPT: &'static str = "📜";
    pub const DOTFILE: &'static str = "🔧";

    // Git related
    pub const GIT: &'static str = "🔀";
    pub const COMMIT: &'static str = "💾";
    pub const BRANCH: &'static str = "🌿";
    pub const MERGE: &'static str = "🔀";
    pub const PULL: &'static str = "⬇️";
    pub const PUSH: &'static str = "⬆️";

    // Status types
    pub const VALID: &'static str = "✅";
    pub const MISSING: &'static str = "❌";
    pub const BROKEN: &'static str = "💥";
    pub const CONFLICT: &'static str = "⚠️";
    pub const INVALID_TARGET: &'static str = "🎯";
    pub const MODIFIED: &'static str = "🔄";

    // UI elements
    pub const ARROW_RIGHT: &'static str = "→";
    pub const ARROW_LEFT: &'static str = "←";
    pub const BULLET: &'static str = "•";
    pub const INDENT: &'static str = "  ";
    pub const TREE_BRANCH: &'static str = "├─";
    pub const TREE_LAST: &'static str = "└─";
    pub const TREE_PIPE: &'static str = "│";

    // Progress
    pub const SPINNER_FRAMES: &'static [&'static str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    pub const PROGRESS_FULL: &'static str = "█";
    pub const PROGRESS_EMPTY: &'static str = "░";

    // Special
    pub const ROCKET: &'static str = "🚀";
    pub const SPARKLES: &'static str = "✨";
    pub const STAR: &'static str = "⭐";
    pub const HEART: &'static str = "❤️";
    pub const FIRE: &'static str = "🔥";
    pub const LIGHTNING: &'static str = "⚡";
    pub const GEAR: &'static str = "⚙️";
    pub const MAGNIFYING_GLASS: &'static str = "🔍";
    pub const LOCK: &'static str = "🔒";
    pub const UNLOCK: &'static str = "🔓";
    pub const KEY: &'static str = "🔑";
}

/// Helper trait to add icon methods to strings
pub trait IconExt {
    fn with_icon(self, icon: &str) -> String;
    fn success_icon(self) -> String;
    fn error_icon(self) -> String;
    fn warning_icon(self) -> String;
    fn info_icon(self) -> String;
}

impl IconExt for &str {
    fn with_icon(self, icon: &str) -> String {
        format!("{} {}", icon, self)
    }

    fn success_icon(self) -> String {
        self.with_icon(Icons::SUCCESS)
    }

    fn error_icon(self) -> String {
        self.with_icon(Icons::ERROR)
    }

    fn warning_icon(self) -> String {
        self.with_icon(Icons::WARNING)
    }

    fn info_icon(self) -> String {
        self.with_icon(Icons::INFO)
    }
}

impl IconExt for String {
    fn with_icon(self, icon: &str) -> String {
        format!("{} {}", icon, self)
    }

    fn success_icon(self) -> String {
        self.with_icon(Icons::SUCCESS)
    }

    fn error_icon(self) -> String {
        self.with_icon(Icons::ERROR)
    }

    fn warning_icon(self) -> String {
        self.with_icon(Icons::WARNING)
    }

    fn info_icon(self) -> String {
        self.with_icon(Icons::INFO)
    }
}