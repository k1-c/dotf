<div align="center">

# ⚙️ dotf ⚡

**Modern Dotfiles Manager**

_Sync your environment / configurations across machines in seconds_

> 🚀 **Alpha Version** - Core functionality implemented. Ready for testing and feedback. Some advanced features may still be in development.

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/dotf.svg?style=for-the-badge)](https://crates.io/crates/dotf)
[![GitHub release](https://img.shields.io/github/release/k1-c/dotf.svg?style=for-the-badge)](https://github.com/k1-c/dotf/releases)

```bash
cargo install dotf
```

_One command to rule them all_ ⚡

</div>

---

**Dotf** transforms how you manage dotfiles. Install and synchronize your development environment from remote repositories with intelligent conflict resolution, automatic dependency management, and beautiful CLI feedback. From zero to configured in minutes.

## ✨ Features

- 🚀 **Modern CLI** - Beautiful, intuitive command-line interface with animations and progress indicators
- 🌿 **Branch Selection** - Choose any branch during initialization with validation
- 🔗 **Smart Symlink Management** - Automatic symbolic link creation with conflict resolution
- 📦 **Dependency Management** - Cross-platform system dependency installation
- 🔄 **Git Integration** - Seamless sync with remote repositories
- 🎯 **Validation** - Repository configuration validation
- 💾 **Backup System** - Safe backup and restore of existing configurations
- 🎨 **Beautiful Output** - Progress bars, animations, and colored terminal output
- ⚙️ **TOML Configuration** - Modern configuration format for better readability

## 🚀 Quick Start

### Installation

```bash
# Install via cargo
cargo install dotf

# Or use the one-liner installer
curl -sSL dotf-install.sh | sh
```

### Initialize from Remote Repository

```bash
# Initialize dotf with a remote repository (with interactive branch selection)
dotf init --repo https://github.com/username/dotfiles.git

# Or initialize without specifying URL (will prompt for URL and branch)
dotf init

# Install system dependencies
dotf install deps

# Install configuration symlinks
dotf install config
```

### Check Status

```bash
# Check current status and sync information
dotf status

# View symlink status
dotf symlinks

# Sync with remote repository
dotf sync
```

## 📖 Usage

### Commands

| Command                 | Description                              |
| ----------------------- | ---------------------------------------- |
| `dotf init`             | Initialize dotf with a remote repository |
| `dotf install deps`     | Install system dependencies              |
| `dotf install config`   | Create configuration symlinks            |
| `dotf install <custom>` | Run custom installation scripts          |
| `dotf status`           | Show repository sync status              |
| `dotf symlinks`         | List symlinks and their status           |
| `dotf symlinks restore` | Restore files from backup                |
| `dotf sync`             | Sync with remote repository              |
| `dotf config`           | View and edit dotf configuration         |

### Workflow

#### 1. Repository Initialization

```bash
# Initialize with remote repository
dotf init --repo https://github.com/username/dotfiles.git
```

This command:

- Detects and displays the repository's default branch
- Prompts for branch selection with validation
- Validates the remote repository structure and configuration
- Creates `~/.dotf/` directory
- Clones the specified branch to `~/.dotf/repo/`
- Creates `~/.dotf/settings.toml` for local configuration

#### 2. Dependency Installation

```bash
# Install system dependencies (languages, tools, etc.)
dotf install deps
```

Executes dependency installation scripts based on your platform.

#### 3. Configuration Installation

```bash
# Install configuration symlinks
dotf install config
```

Creates symbolic links according to your configuration. If conflicts exist:

- Prompts to backup existing files to `~/.dotf/backups/`
- Option to abort installation
- Safe conflict resolution

#### 4. Custom Installations

```bash
# Run custom installation scripts
dotf install vim-plugins
dotf install zsh-setup
```

Execute custom installation scripts defined in your configuration.

## 🔧 Repository Configuration

Your dotfiles repository should contain a `dotf.toml` configuration file:

```toml
[symlinks]
# Source (in repo) -> Target (on system)
"nvim" = "~/.config/nvim"
"tmux/tmux.conf" = "~/.tmux.conf"
"zsh/zshrc" = "~/.zshrc"
"git/gitconfig" = "~/.gitconfig"
"alacritty/alacritty.yml" = "~/.config/alacritty/alacritty.yml"

[scripts.deps]
# Dependency installation scripts
macos = "scripts/install-deps-macos.sh"
linux = "scripts/install-deps-linux.sh"
windows = "scripts/install-deps-windows.ps1"

[scripts.custom]
# Custom installation scripts
vim-plugins = "scripts/install-vim-plugins.sh"
zsh-setup = "scripts/setup-zsh.sh"
font-install = "scripts/install-fonts.sh"
```

### Example Repository Structure

```
my-dotfiles/
├── dotf.toml              # Configuration file
├── README.md
├── nvim/                  # Neovim configuration
│   ├── init.vim
│   └── plugins/
├── tmux/                  # Tmux configuration
│   └── tmux.conf
├── zsh/                   # Zsh configuration
│   ├── zshrc
│   └── aliases.zsh
├── git/                   # Git configuration
│   └── gitconfig
├── alacritty/            # Terminal configuration
│   └── alacritty.yml
├── macos/                # macOS specific configs
│   ├── yabai/
│   └── skhd/
├── linux/                # Linux specific configs
│   ├── i3/
│   └── rofi/
└── scripts/              # Installation scripts
    ├── install-deps-macos.sh
    ├── install-deps-linux.sh
    ├── install-vim-plugins.sh
    └── setup-zsh.sh
```

## 🔧 Local Configuration

Dotf creates `~/.dotf/settings.toml` for local configuration:

```toml
last_sync = 2024-01-15T10:30:00Z
initialized_at = 2024-01-15T09:00:00Z

[repository]
remote = "https://github.com/username/dotfiles.git"
branch = "main"
local = "/home/user/.dotf/repo"
```

## 🎯 Status and Monitoring

### Status Output

```bash
$ dotf status

📁 Repository: https://github.com/username/dotfiles.git
├── 🌿 Branch: main
├── 🔄 Status: 2 commits behind origin/main
├── 📂 Local: ~/.dotf/repo
└── ⏰ Last sync: 2 hours ago

📁 Symlinks: 8 total, 6 active, 2 conflicts
📦 Dependencies: 12 installed, 1 missing
🔧 Custom scripts: 3 available
```

### Symlinks Status

```bash
$ dotf symlinks

📁 Configuration Symlinks
├── ✅ ~/.zshrc → ~/.dotf/repo/zsh/zshrc
├── ✅ ~/.tmux.conf → ~/.dotf/repo/tmux/tmux.conf
├── ✅ ~/.config/nvim → ~/.dotf/repo/nvim
├── ❌ ~/.gitconfig (conflict: exists, not symlinked)
├── ⚠️  ~/.config/alacritty (missing target)
└── 🔄 ~/.vimrc (backed up, symlink active)

💾 Backups available: 3 files
```

### Shell Integration

Add to your shell configuration for sync monitoring:

```bash
# .zshrc or .bashrc
# Check dotf status on shell startup
if command -v dotf >/dev/null 2>&1; then
    dotf status --quiet
fi
```

## 🔄 Sync and Updates

### Sync with Remote

```bash
# Check for updates
dotf status

# Sync with remote repository
dotf sync

# Force sync (override local changes)
dotf sync --force
```

### Backup and Restore

```bash
# List available backups
dotf symlinks restore --list

# Restore specific file from backup
dotf symlinks restore ~/.gitconfig

# Restore all backed up files
dotf symlinks restore --all
```

## 🎨 Configuration Management

### View Configuration

```bash
# View current dotf configuration
dotf config

# View repository configuration
dotf config --repo

# Edit local settings
dotf config --edit
```

### Custom Installation Scripts

Create executable scripts for complex setup tasks:

```bash
# scripts/install-vim-plugins.sh
#!/bin/bash
echo "Installing Vim plugins..."
vim +PlugInstall +qall

# scripts/setup-zsh.sh
#!/bin/bash
echo "Setting up Zsh environment..."
if [ ! -d ~/.oh-my-zsh ]; then
    sh -c "$(curl -fsSL https://raw.github.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"
fi

git clone https://github.com/zsh-users/zsh-autosuggestions ~/.oh-my-zsh/custom/plugins/zsh-autosuggestions
```

Run custom scripts:

```bash
dotf install vim-plugins
dotf install setup-zsh
```

## 📋 Common Workflows

### Initial Setup on New Machine

```bash
# 1. Initialize with your dotfiles repository (will prompt for branch selection)
dotf init https://github.com/myuser/dotfiles.git

# 2. Install system dependencies
dotf install deps

# 3. Install configuration symlinks
dotf install config

# 4. Run custom setup scripts
dotf install vim-plugins
dotf install zsh-setup

# 5. Check final status
dotf status
```

The init process will:

- Detect the repository's default branch (e.g., "main", "master")
- Prompt you to select a branch with the default pre-filled
- Validate that the selected branch exists
- Clone from the chosen branch

### Daily Sync Workflow

```bash
# Check for updates (add to shell startup)
dotf status

# Sync when updates available
dotf sync

# Check symlink health
dotf symlinks
```

### Backup and Recovery

```bash
# Before major changes, ensure backups
dotf symlinks restore --list

# If something breaks, restore from backup
dotf symlinks restore ~/.zshrc
```

## 🚧 Development

**🚀 Alpha Version** - Core functionality implemented and ready for testing. Feedback and contributions welcome!

### Building from Source

```bash
git clone https://github.com/k1-c/dotf.git
cd dotf
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📝 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by GNU Stow, Dotbot, and other dotfile management tools
- Built with ❤️ using Rust

---

**Made with 🦀 Rust**
