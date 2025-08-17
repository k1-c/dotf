# Dotf Specification

## Overview

Dotfは、リモートリポジトリからdotfilesを取得・管理・同期するためのコマンドラインツールです。
リモートの更新チェックやリモート・ローカルでの変更の同期、シンボリックリンクの自動管理、依存関係のインストール、安全なバックアップ機能を提供します。

## 全体仕様

### アーキテクチャ

- **設定ディレクトリ**: `~/.dotf/`
- **デフォルトのリポジトリクローン先**: `~/.dotf/repo/`
- **ローカル設定**: `~/.dotf/settings.json`
- **バックアップ**: `~/.dotf/backups/{timestamp}/`

### 設定ファイル

#### dotf.toml（リポジトリ内）
リポジトリのルートに配置するか、.dotf/dotf.tomlとして配置します。

```toml
[symlinks]
"システム内パス" = "リポジトリ内パス"

[scripts.deps]
macos = "スクリプトパス"
linux = "スクリプトパス"

[scripts.custom]
名前 = "スクリプトパス"
```

#### settings.json（ローカル）
ローカル設定ファイルは、dotfの初期化時に自動生成されます。
~/.dotf/settings.jsonとして配置されます。

```json
{
  "repository": {
    "url": "リポジトリURL",
    "path": "リポジトリローカルパス",
    "branch": "ブランチ名"
  },
  "backup": {
    "enabled": true,
    "path": "バックアップディレクトリ"
  },
  "sync": {
    "auto_check": false,
    "merge_strategy": "rebase"
  }
}
```

## コマンド仕様

### `dotf init`

リモートリポジトリからdotfを初期化します。

**機能**:
- リモートリポジトリの指定（コマンドライン引数または対話的入力）
- 設定ファイル（dotf.toml）の検証
- ローカル環境の構築

**コマンド形式**:
```bash
# オプション引数でURL指定
dotf init --repo <repository_url>

# 対話的入力（引数なし）
dotf init
```

**処理内容**:
1. リポジトリURL取得（`--repo`指定または対話的入力）
2. リポジトリURLの妥当性確認
3. リモートリポジトリから`dotf.toml`を取得・検証
4. `~/.dotf/`ディレクトリ作成
5. リポジトリを`~/.dotf/repo/`にクローン
6. `~/.dotf/settings.json`を作成

**対話的入力例**:
```
$ dotf init
? Repository URL: https://github.com/username/dotfiles.git
🔍 Validating repository...
✅ Repository validation successful
📁 Creating ~/.dotf/ directory...
🔄 Cloning repository...
✅ Initialization complete!
```

**オプション**:
- `--repo <url>`: リモートリポジトリのURL

**エラーケース**:
- 無効なURL
- アクセス不可能なリポジトリ
- `dotf.toml`の不存在・不正

### `dotf install deps`

システム依存関係をインストールします。

**機能**:
- プラットフォーム検出（macOS / Linux）
- 設定ファイルで指定されたスクリプトの実行

**対応プラットフォーム**:
- **macOS**: `[scripts.deps]`の`macos`で指定されたスクリプトを実行
- **Linux**: `[scripts.deps]`の`linux`で指定されたスクリプトを実行

**処理内容**:
1. 現在のプラットフォームを検出
2. プラットフォームに対応する設定を`[scripts.deps]`から取得
3. 指定されたスクリプトファイルの存在確認
4. スクリプトを実行

**設定例**:
```toml
[scripts.deps]
macos = "scripts/install-deps-macos.sh"
linux = "scripts/install-deps-linux.sh"
```

**エラーケース**:
- 未対応プラットフォーム（Windows等）
- 対応するプラットフォーム設定の不存在
- スクリプトファイルの不存在
- スクリプト実行権限の不足
- スクリプト実行エラー

### `dotf install config`

設定ファイルのシンボリックリンクを作成します。

**機能**:
- シンボリックリンクの作成
- 既存ファイルとの競合処理
- バックアップ機能

**処理内容**:
1. `[symlinks]`設定の読み込み
2. プラットフォーム固有設定の適用
3. 各ファイルの競合チェック
4. 競合時のユーザー選択（バックアップ/中断）
5. シンボリックリンクの作成

**エラーケース**:
- ソースファイルの不存在
- ターゲットディレクトリの作成失敗
- 権限エラー

### `dotf install <custom>`

カスタムインストールスクリプトを実行します。

**機能**:
- 指定されたカスタムスクリプトの実行
- 実行順序の制御

**処理内容**:
1. `[scripts.custom]`から指定スクリプトを検索
2. スクリプトファイルの存在確認
3. 実行権限の確認
4. スクリプト実行

**エラーケース**:
- 存在しないスクリプト名
- スクリプトファイルの不存在
- 実行権限の不足
- スクリプト実行エラー

### `dotf status`

現在の状態を表示します。

**機能**:
- リポジトリ同期状態の表示
- シンボリックリンク状態の表示
- 依存関係状態の表示

**表示内容**:
- リポジトリ情報（URL、ブランチ、同期状態）
- シンボリックリンク統計（総数、アクティブ、競合）
- 依存関係統計（インストール済み、未インストール）
- 利用可能なカスタムスクリプト数

**オプション**:
- `--quiet`: 簡潔な表示

### `dotf symlinks`

シンボリックリンクの詳細状態を一覧表示します。

**機能**:
- 各シンボリンクの状態表示
- 競合やエラーの詳細表示

**表示内容**:
- ✅ 正常なシンボリンク
- ❌ 競合（既存ファイル）
- ⚠️ エラー（ソース不存在など）
- 🔄 バックアップ済み

### `dotf symlinks restore`

バックアップからファイルを復元します。

**機能**:
- バックアップファイルの復元
- 復元対象の選択

**オプション**:
- `--list`: 利用可能なバックアップ一覧
- `--all`: 全ファイルの一括復元
- `<filepath>`: 特定ファイルの復元

### `dotf sync`

リモートリポジトリと同期します。

**機能**:
- リモートからの変更取得
- ローカルリポジトリの更新
- 必要に応じたシンボリンクの再作成

**処理内容**:
1. リモートリポジトリの変更確認
2. ローカル変更の確認
3. マージ戦略に基づく同期
4. 設定変更時のシンボリンク更新

**オプション**:
- `--force`: ローカル変更を無視して強制同期

### `dotf config`

設定の表示・編集を行います。

**機能**:
- 設定情報の表示
- ローカル設定の編集

**オプション**:
- `--repo`: リポジトリ設定（dotf.toml）を表示
- `--edit`: ローカル設定（settings.json）を編集

### `dotf schema`

dotf.toml設定ファイルの管理を行います。

#### `dotf schema init`

dotfiles リポジトリ内で `dotf.toml` テンプレートファイルを生成します。

**機能**:
- dotf.toml テンプレートの生成

**コマンド形式**:
```bash
# テンプレート生成
dotf schema init
```

**処理内容**:
1. 既存の `dotf.toml` の存在確認
2. `dotf.toml`が存在しない場合のみ、テンプレートの生成

**生成されるテンプレート**:
```toml
[symlinks]
# {Source path} = {Target path}
# Example:
# "zsh/.zshrc" = "~/.zshrc"
# "git/.gitconfig" = ""~/.gitconfig"
# "nvim" = "~/.config/nvim"

[scripts.deps]
# Platform-specific dependency installation scripts
# Example:
# macos = "scripts/install-deps-macos.sh"
# linux = "scripts/install-deps-linux.sh"

[scripts.custom]
# Custom installation scripts
# setup-vim = "scripts/setup-vim-plugins.sh"
# install-fonts = "scripts/install-fonts.sh"
```

**エラーケース**:
- `dotf.toml` が既に存在する
- ファイル書き込み権限の不足
- ディスク容量不足

#### `dotf schema test`

現在のディレクトリまたは指定された `dotf.toml` の構文と構造を検証します。

**機能**:
- TOML構文の検証
- dotfスキーマ仕様との適合性チェック
- シンボリックリンク設定の妥当性検証
- スクリプトファイルの存在確認

**コマンド形式**:
```bash
# 現在のディレクトリのdotf.tomlを検証
dotf schema test

# ファイルパスを指定して検証
dotf schema test --file path/to/dotf.toml

# Short Option
dotf schema test -f path/to/dotf.toml

# エラー時に異常終了しない（デフォルトは異常終了）
dotf schema test --ignore-errors
```

**処理内容**:
1. 指定された `dotf.toml` ファイルの存在確認
2. TOML構文解析
3. 必須セクションの存在確認
4. シンボリックリンク設定の妥当性チェック
   - ソースパスの形式検証
   - ソースパスの存在確認
   - ターゲットパスの形式検証
   - パス文字列の空白チェック
5. スクリプト設定の検証
   - スクリプトファイルの存在確認

**出力例（成功）**:
```
$ dotf schema validate
🔍 Validating dotf.toml...

✅ TOML syntax: Valid
✅ Schema compliance: Valid
✅ Symlinks configuration: 12 entries, all valid
✅ Scripts configuration: 3 entries, all files exist

🎉 dotf.toml validation successful!
```

**出力例（エラーあり）**:
```
$ dotf schema validate
🔍 Validating dotf.toml...

✅ TOML syntax: Valid
❌ Schema compliance: Issues found

🚨 Validation errors:
   Line 15: [symlinks] Empty source path: "" = ".vimrc"
   Line 23: [scripts.deps] Missing script file: scripts/install-deps-linux.sh
   Line 31: [platform.macos.symlinks] Invalid target path: "invalid-path"

❌ Validation failed with 3 errors.
```

**オプション**:
- `--file <path>`: 検証対象ファイルを指定（デフォルト: ./dotf.toml）
- `--ignore-errors`: 検証エラーがある場合でもゼロコードで終了
- `--quiet`: エラーと警告のみ表示

**検証項目**:

1. **構文検証**:
   - TOML形式の妥当性

2. **構造検証**:
   - 必須セクション（`[symlinks]`）の存在
   - セクション名の妥当性

3. **シンボリックリンク検証**:
   - ソースパス・ターゲットパスの非空文字列
   - パスの形式妥当性（相対パス、絶対パス、チルダ展開）
   - 重複エントリの検出

4. **スクリプト検証**:
   - スクリプトファイルの存在

**終了ステータス**:
- `0`: 検証成功
- `1`: 検証エラー (--ignore-errorsオプションの場合は`0`)
- `2`: ファイル不存在、権限エラーなどのシステムエラー

**前提条件**:
- `dotf.toml` ファイルが存在する
- ファイルへの読み取り権限がある

**エラーケース**:
- `dotf.toml` ファイルが存在しない
- ファイル読み取り権限の不足
- TOML構文エラー
- 必須セクションの不存在
- 無効な設定値

## テストケース

### スキーマ管理テスト

#### dotf schema init テスト

**正常なテンプレート生成**
- 前提: dotf.toml が存在しないディレクトリ
- 実行: `dotf schema init`
- 期待: 固定テンプレート内容で dotf.toml が生成される

**既存ファイルが存在する場合**
- 前提: dotf.toml が既に存在する
- 実行: `dotf schema init`
- 期待: エラーメッセージ表示、既存ファイル保持

**書き込み権限不足**
- 前提: 書き込み権限のないディレクトリ
- 実行: `dotf schema init`
- 期待: 権限エラー表示、ファイル未作成

**ディスク容量不足**
- 前提: ディスク容量が不足している環境
- 実行: `dotf schema init`
- 期待: 容量不足エラー表示、ファイル未作成

#### dotf schema test テスト

**正常な設定ファイル検証**
- 前提: 構文的に正しく、すべてのソースファイルとスクリプトが存在するdotf.toml
- 実行: `dotf schema test`
- 期待: 検証成功メッセージ、終了ステータス0

**TOML構文エラー**
- 前提: 構文的に無効なTOMLファイル（例: 不正な引用符、閉じ括弧なし）
- 実行: `dotf schema test`
- 期待: TOML構文エラー報告、終了ステータス1

**必須セクション不存在**
- 前提: [symlinks]セクションが存在しないdotf.toml
- 実行: `dotf schema test`
- 期待: 必須セクション不存在エラー、終了ステータス1

**空のソースパス**
- 前提: シンボリックリンク設定で空のソースパスを含む設定
  ```toml
  [symlinks]
  "" = "~/.vimrc"
  ```
- 実行: `dotf schema test`
- 期待: 空ソースパスエラー報告、終了ステータス1

**空のターゲットパス**
- 前提: シンボリックリンク設定で空のターゲットパスを含む設定
  ```toml
  [symlinks]
  "vim/.vimrc" = ""
  ```
- 実行: `dotf schema test`
- 期待: 空ターゲットパスエラー報告、終了ステータス1

**存在しないソースファイル**
- 前提: 存在しないファイルをソースパスに指定した設定
  ```toml
  [symlinks]
  "nonexistent/.vimrc" = "~/.vimrc"
  ```
- 実行: `dotf schema test`
- 期待: ソースファイル不存在エラー報告、終了ステータス1

**存在しないスクリプトファイル**
- 前提: 存在しないスクリプトファイルを参照する設定
  ```toml
  [scripts.deps]
  linux = "scripts/nonexistent.sh"
  ```
- 実行: `dotf schema test`
- 期待: スクリプトファイル不存在エラー報告、終了ステータス1

**重複シンボリックリンクエントリ**
- 前提: 同じターゲットパスに複数のソースが指定された設定
  ```toml
  [symlinks]
  "vim/.vimrc" = "~/.vimrc"
  "backup/.vimrc" = "~/.vimrc"
  ```
- 実行: `dotf schema test`
- 期待: 重複エントリエラー報告、終了ステータス1

**無効なパス形式**
- 前提: 無効な文字を含むパス設定
  ```toml
  [symlinks]
  "vim/.vimrc" = "~/invalid\0path"
  ```
- 実行: `dotf schema test`
- 期待: 無効パス形式エラー報告、終了ステータス1

**ファイル指定オプション**
- 前提: 指定パスに正しいdotf.tomlが存在
- 実行: `dotf schema test --file /path/to/dotf.toml`
- 期待: 指定ファイルの検証実行、成功メッセージ

**ファイル指定オプション（短縮形）**
- 前提: 指定パスに正しいdotf.tomlが存在
- 実行: `dotf schema test -f /path/to/dotf.toml`
- 期待: 指定ファイルの検証実行、成功メッセージ

**指定ファイル不存在**
- 前提: 存在しないファイルパスを指定
- 実行: `dotf schema test --file /nonexistent/dotf.toml`
- 期待: ファイル不存在エラー、終了ステータス2

**読み取り権限不足**
- 前提: 読み取り権限のないdotf.toml
- 実行: `dotf schema test`
- 期待: 権限エラー、終了ステータス2

**エラー無視オプション**
- 前提: 検証エラーを含むdotf.toml
- 実行: `dotf schema test --ignore-errors`
- 期待: エラー報告されるが終了ステータス0

**静寂モード**
- 前提: エラーと正常メッセージの両方がある設定
- 実行: `dotf schema test --quiet`
- 期待: エラーメッセージのみ表示、正常メッセージは非表示

**デフォルトファイル不存在**
- 前提: カレントディレクトリにdotf.tomlが存在しない
- 実行: `dotf schema test`
- 期待: デフォルトファイル不存在エラー、終了ステータス2

### 初期化テスト

**正常な初期化（オプション指定）**
- 前提: 有効なリモートリポジトリURL
- 実行: `dotf init --repo https://github.com/user/dotfiles.git`
- 期待: `~/.dotf/`構築、リポジトリクローン、設定ファイル作成

**正常な初期化（対話的入力）**
- 前提: 有効なリモートリポジトリURL
- 実行: `dotf init` → URL入力プロンプト
- 期待: 対話的にURL入力、`~/.dotf/`構築、リポジトリクローン、設定ファイル作成

**無効なリポジトリ**
- 前提: 存在しないリポジトリURL
- 実行: `dotf init --repo https://github.com/invalid/repo.git`
- 期待: エラーメッセージ、ローカルファイル未作成

**対話的入力でのキャンセル**
- 前提: 対話的入力でユーザーがキャンセル
- 実行: `dotf init` → Ctrl+C または空入力
- 期待: 処理中断、ローカルファイル未作成

**設定ファイル不備**
- 前提: `dotf.toml`が存在しないリポジトリ
- 実行: `dotf init <repository_without_config>`
- 期待: 設定ファイル不備エラー、初期化中断

### 依存関係インストールテスト

**正常な依存関係インストール（macOS）**
- 前提: 初期化済み、macOS環境、有効なスクリプト設定
- 実行: `dotf install deps`
- 期待: `[scripts.deps]`の`macos`スクリプト実行

**正常な依存関係インストール（Linux）**
- 前提: 初期化済み、Linux環境、有効なスクリプト設定
- 実行: `dotf install deps`
- 期待: `[scripts.deps]`の`linux`スクリプト実行

**未対応プラットフォーム**
- 前提: Windows環境
- 実行: `dotf install deps`
- 期待: 未対応プラットフォームエラー

**依存関係スクリプト設定不在**
- 前提: プラットフォーム対応設定が`[scripts.deps]`に存在しない
- 実行: `dotf install deps`
- 期待: 設定不在エラー

### 設定インストールテスト

**正常なシンボリンク作成**
- 前提: 競合のない環境
- 実行: `dotf install config`
- 期待: 設定通りのシンボリンク作成

**ファイル競合の処理**
- 前提: 既存ファイルが存在
- 実行: `dotf install config`
- 期待: 競合選択肢の提示、バックアップまたは中断

**シンボリンク状態の表示**
- 前提: 一部シンボリンクが作成済み
- 実行: `dotf symlinks`
- 期待: 各シンボリンクの状態表示

### カスタムインストールテスト

**正常なカスタムスクリプト実行**
- 前提: 有効なカスタムスクリプト設定
- 実行: `dotf install vim-plugins`
- 期待: 指定スクリプト実行

**存在しないカスタムスクリプト**
- 前提: 設定にないスクリプト名
- 実行: `dotf install non-existent-script`
- 期待: エラーメッセージ

### ステータス表示テスト

**正常なステータス表示**
- 前提: 初期化済み環境
- 実行: `dotf status`
- 期待: リポジトリ情報、シンボリンク状況、依存関係状況の表示

**未初期化環境**
- 前提: dotfが初期化されていない
- 実行: `dotf status`
- 期待: 未初期化メッセージ

### 同期テスト

**正常な同期**
- 前提: リモートに新しいコミットが存在
- 実行: `dotf sync`
- 期待: ローカルリポジトリ更新、必要に応じたシンボリンク再作成

**競合のある同期**
- 前提: ローカルに未コミットの変更
- 実行: `dotf sync`
- 期待: 競合警告、`--force`オプション案内

### バックアップ・復元テスト

**バックアップからの復元**
- 前提: バックアップが存在
- 実行: `dotf symlinks restore ~/.zshrc`
- 期待: 指定ファイルのバックアップからの復元

**バックアップ一覧表示**
- 前提: 複数のバックアップが存在
- 実行: `dotf symlinks restore --list`
- 期待: 利用可能なバックアップの一覧表示
