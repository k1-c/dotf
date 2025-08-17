# dotf.toml 設定仕様書

本ドキュメントでは、dotfツールの設定ファイル `dotf.toml` の詳細な仕様について説明します。

## 概要

`dotf.toml` は、dotfilesリポジトリのルートディレクトリまたは `.dotf/` サブディレクトリに配置する設定ファイルです。TOML形式で記述し、シンボリックリンク設定、スクリプト設定、プラットフォーム固有設定を定義します。

## 基本構造

```toml
[symlinks]
# 基本シンボリックリンク設定

[scripts.deps]
# 依存関係インストールスクリプト

[scripts.custom]
# カスタムスクリプト

[platform.linux]
# Linux固有設定

[platform.macos]
# macOS固有設定
```

## セクション詳細

### `[symlinks]` - シンボリックリンク設定

dotfilesのシンボリックリンク作成規則を定義します。

#### 基本形式

```toml
[symlinks]
"ターゲットパス" = "ソースパス"
```

- **ターゲットパス**: シンボリックリンクを作成する場所（リンク先）
- **ソースパス**: リポジトリ内の実際のファイル/ディレクトリ（リンク元）

#### パス仕様

##### ターゲットパス（キー）

```toml
[symlinks]
# ホームディレクトリからの相対パス（推奨）
"~/.vimrc" = "vim/.vimrc"
"~/.config/nvim" = "nvim"

# 絶対パス
"/etc/hosts" = "system/hosts"
```

- `~` でホームディレクトリを表現
- 絶対パス指定も可能
- 親ディレクトリが存在しない場合は自動作成

##### ソースパス（値）

```toml
[symlinks]
# 相対パス（リポジトリルートからの相対）
"~/.vimrc" = "config/vim/.vimrc"
"~/.zshrc" = "zsh/.zshrc"

# 絶対パス
"~/.bashrc" = "/path/to/external/bashrc"
```

- 相対パス: リポジトリルートからの相対パス
- 絶対パス: `/` で始まる場合は絶対パス扱い

#### 設定例

```toml
[symlinks]
# 設定ファイル
"~/.vimrc" = "vim/.vimrc"
"~/.zshrc" = "zsh/.zshrc"
"~/.gitconfig" = "git/.gitconfig"

# ディレクトリ全体
"~/.config/nvim" = "nvim"
"~/.ssh" = "ssh"

# アプリケーション設定
"~/.config/alacritty/alacritty.yml" = "alacritty/alacritty.yml"
"~/.tmux.conf" = "tmux/.tmux.conf"
```

### `[scripts.deps]` - 依存関係スクリプト

プラットフォーム別の依存関係インストールスクリプトを定義します。

#### 基本形式

```toml
[scripts.deps]
macos = "scripts/install-deps-macos.sh"
linux = "scripts/install-deps-linux.sh"
```

#### サポートプラットフォーム

- `macos`: macOS環境用スクリプト
- `linux`: Linux環境用スクリプト

#### 設定例

```toml
[scripts.deps]
# macOS用（Homebrew使用）
macos = "scripts/install-homebrew.sh"

# Linux用（apt/yum使用）
linux = "scripts/install-apt.sh"
```

#### スクリプト要件

- 実行権限を持つシェルスクリプト
- リポジトリルートからの相対パス
- エラー時は非ゼロの終了ステータスを返す

### `[scripts.custom]` - カスタムスクリプト

任意の名前で実行可能なカスタムスクリプトを定義します。

#### 基本形式

```toml
[scripts.custom]
スクリプト名 = "スクリプトパス"
```

#### 設定例

```toml
[scripts.custom]
setup-vim = "scripts/setup-vim-plugins.sh"
install-fonts = "scripts/install-fonts.sh"
configure-git = "scripts/configure-git.sh"
setup-python = "scripts/setup-python-env.sh"
```

#### 実行方法

```bash
# 特定のカスタムスクリプトを実行
dotf install setup-vim

# すべてのカスタムスクリプトを対話的に実行
dotf install all
```

### `[platform.<os>]` - プラットフォーム固有設定

OS固有の設定を定義します。基本設定に加えて適用されます。

#### サポートOS

- `linux`: Linux環境固有設定
- `macos`: macOS環境固有設定

#### プラットフォーム固有シンボリックリンク

```toml
[platform.linux]
[platform.linux.symlinks]
"~/.config/fontconfig/fonts.conf" = "linux/fontconfig/fonts.conf"

[platform.macos]
[platform.macos.symlinks]
"~/Library/Preferences/com.apple.Terminal.plist" = "macos/Terminal.plist"
```

#### 設定例

```toml
# 基本設定（全プラットフォーム共通）
[symlinks]
"~/.vimrc" = "vim/.vimrc"
"~/.gitconfig" = "git/.gitconfig"

# Linux固有設定
[platform.linux.symlinks]
"~/.config/fontconfig/fonts.conf" = "linux/fonts.conf"
"~/.xprofile" = "linux/.xprofile"

# macOS固有設定
[platform.macos.symlinks]
"~/Library/Application Support/Code/User/settings.json" = "vscode/settings.json"
"~/.yabairc" = "macos/.yabairc"
```

## 完全な設定例

```toml
# 基本シンボリックリンク設定
[symlinks]
"~/.vimrc" = "vim/.vimrc"
"~/.zshrc" = "zsh/.zshrc"
"~/.gitconfig" = "git/.gitconfig"
"~/.config/nvim" = "nvim"
"~/.tmux.conf" = "tmux/.tmux.conf"

# 依存関係スクリプト
[scripts.deps]
macos = "scripts/install-homebrew-packages.sh"
linux = "scripts/install-apt-packages.sh"

# カスタムスクリプト
[scripts.custom]
setup-vim = "scripts/setup-vim-plugins.sh"
install-fonts = "scripts/install-nerd-fonts.sh"
configure-git = "scripts/configure-git-user.sh"
setup-python = "scripts/setup-python-environment.sh"

# Linux固有設定
[platform.linux.symlinks]
"~/.config/fontconfig/fonts.conf" = "linux/fontconfig/fonts.conf"
"~/.xprofile" = "linux/.xprofile"
"~/.config/i3/config" = "linux/i3/config"

# macOS固有設定
[platform.macos.symlinks]
"~/Library/Application Support/Code/User/settings.json" = "vscode/settings.json"
"~/Library/Application Support/Code/User/keybindings.json" = "vscode/keybindings.json"
"~/.yabairc" = "macos/.yabairc"
"~/.skhdrc" = "macos/.skhdrc"
```

## 注意事項とベストプラクティス

### パス指定の注意点

1. **ターゲットパスの`~`展開**: `~/.vimrc` は正しく展開されます
2. **ソースパスに`~`は使用不可**: `~/config/.vimrc` ではなく `config/.vimrc` を使用
3. **相対パス推奨**: ソースパスはリポジトリルートからの相対パスを推奨

### よくある間違い

```toml
# ❌ 間違い: ソースパスに ~ を使用
"~/.vimrc" = "~/dotfiles/vim/.vimrc"

# ✅ 正しい: 相対パスを使用
"~/.vimrc" = "vim/.vimrc"

# ❌ 間違い: キーと値が逆
"vim/.vimrc" = "~/.vimrc"

# ✅ 正しい: ターゲット = ソース
"~/.vimrc" = "vim/.vimrc"
```

### ディレクトリ構造の推奨例

```
dotfiles/
├── dotf.toml
├── scripts/
│   ├── install-homebrew-packages.sh
│   ├── install-apt-packages.sh
│   └── setup-vim-plugins.sh
├── vim/
│   └── .vimrc
├── zsh/
│   └── .zshrc
├── nvim/
│   ├── init.lua
│   └── plugins/
├── linux/
│   ├── fontconfig/
│   └── i3/
└── macos/
    ├── .yabairc
    └── .skhdrc
```

### エラー処理

- ソースファイルが存在しない場合、`dotf install config` は失敗
- 既存ファイルとの競合時は、対話的にバックアップまたはスキップを選択
- スクリプト実行エラー時は、詳細なエラーメッセージを表示

### 検証方法

```bash
# 設定ファイルの構文チェック
dotf config --repo

# シンボリックリンクの状態確認
dotf symlinks

# 全体的な状態確認
dotf status
```