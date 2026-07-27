# gh-review-watcher

GitHub上で自分にレビュー依頼された / アサインされたPRをポーリングで監視するTUIアプリ。

## 機能

- **レビュー依頼PR** (`--review-requested=@me`) と **アサインPR** (`--assignee=@me`) を一覧表示
- 新規PR検出時に任意のコマンドを実行（macOS通知、zellijペイン起動など）
- Enter押下で選択PRに対するアクション実行
- 時間ベースのフィルタ（All / 24h / 7d）でアクティブなPRに絞り込み
- リフレッシュ中のインジケーター表示

## キーバインド

| キー | アクション |
|------|-----------|
| `q` | 終了 |
| `j` / `k` | 上下移動 |
| `Enter` | 選択PRのアクション実行 (`[on_select]`) |
| `a` | 手動アクションメニューを開く (`[[actions]]`) |
| `r` | 手動リフレッシュ |
| `Tab` | フィルタ切り替え (All → 24h → 7d) |

メニュー表示中: `j`/`k` で移動、`Enter` で実行、`Esc`/`a`/`q` で閉じる。

## 設定

`~/.config/gh-review-watcher/config.toml`:

```toml
# ポーリング間隔（秒）
interval = 120

# 新規PR検出時: macOS通知
[[on_new_pr]]
name = "notify"
command = """/Applications/Utilities/Notifier.app/Contents/MacOS/Notifier \
  --type banner --title "PR Review Request" \
  --subtitle "{repo} #{number}" \
  --message "{title} by @{author}" \
  --sound default"""

# 新規PR検出時: zellijでfloatingペインを開いてレビュー開始
[[on_new_pr]]
name = "review-pane"
command = """zellij action new-pane --floating --name "Review: {repo}#{number}" -- \
  claude --dangerously-skip-permissions -p \
  "PR #{number} in {repo} をレビューしてください。URL: {url}"""

# Enter押下時: ブラウザでPRを開く
[on_select]
command = "open {url}"

# 手動アクション: `a` でメニューを開き、選択中PRに対して実行
[[actions]]
name = "Rebase"
command = "gh pr update-branch {number} -R {repo} --rebase"

[[actions]]
name = "Approve"
command = "gh pr review {number} -R {repo} --approve"

[[actions]]
name = "Copy URL"
command = "printf %s {url} | pbcopy"
```

### フックの種類

| フック | タイミング | 対象 |
|--------|-----------|------|
| `[[on_new_pr]]` | 新規PR検出時 | 検出されたPR |
| `[[on_poll]]` | 毎ポーリング | 追跡中の全PR |
| `[[on_remove]]` | PRがリストから消えた時（マージ・クローズ・レビュー解除等） | 消えたPR |
| `[on_select]` | Enter押下時（既定の1アクション） | 選択中のPR |
| `[[actions]]` | `a` で開くメニューから選択実行（複数可） | 選択中のPR |

### テンプレート変数

| 変数 | 内容 |
|------|------|
| `{repo}` | リポジトリ名 (owner/repo) |
| `{number}` | PR番号 |
| `{title}` | PRタイトル |
| `{author}` | PR作成者 |
| `{url}` | PRのURL |
| `{labels}` | PRのラベル（カンマ区切り） |

### 設定例

[examples/config.toml](examples/config.toml) に、macOS + Zellij + Claude Code を使ったPRレビュー自動化の設定例があります。

## インストール

### Nix Flake

```nix
# flake.nix の inputs に追加
gh-review-watcher = {
  url = "github:EdV4H/gh-review-watcher";
  inputs.nixpkgs.follows = "nixpkgs";
};
```

### Cargo

```bash
cargo install --path .
```

## ビルド

```bash
nix build
# or
cargo build --release
```

## 前提条件

- [GitHub CLI (`gh`)](https://cli.github.com/) がインストール・認証済みであること
