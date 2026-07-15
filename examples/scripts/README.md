# examples/scripts

`examples/config.toml` の hook から呼ばれるヘルパースクリプトの例です。
そのまま使う必要はなく、自分のワークフローに合わせて改変する土台としてどうぞ。

## スクリプト

| スクリプト | 呼ばれる hook | 役割 |
|---|---|---|
| `review-pr.sh` | `on_new_pr` (review-tab) | Zellij タブ内で PR をレビュー。`claude -p /review` で分析し、固定テンプレに整形して表示、`[a]pprove / [c]omment / [d]iscuss / [o]pen / [q]uit` を対話選択 |
| `close-merged-review-tab.sh` | `on_remove` | マージ/クローズ等で消えた PR のレビュータブを Zellij から閉じる |

## 設置

`config.toml` は拡張子なしのコマンド名 (`review-pr` / `close-merged-review-tab`) で
参照しているので、PATH 上にその名前で置く（シンボリックリンク推奨）:

```sh
mkdir -p ~/.local/bin
ln -sf "$PWD/review-pr.sh"               ~/.local/bin/review-pr
ln -sf "$PWD/close-merged-review-tab.sh" ~/.local/bin/close-merged-review-tab
# ~/.local/bin が PATH に入っていること
```

## 依存

- [Zellij](https://zellij.dev/) — タブ操作 (`zellij action ...`)
- [Claude Code](https://claude.com/claude-code) — `claude` CLI。`review-pr.sh` は
  スラッシュコマンド `/review`、`config.toml` の yolo-review hook は `/yolo-review`
  を使う（`/yolo-review` は各自で用意する Claude Code skill / command）
- `gh` (GitHub CLI), `jq`

いずれも `{repo}` `{number}` `{url}` などの値は gh-review-watcher が hook 実行時に
展開して引数で渡す。スクリプト自体にトークン等の秘密は含まれていない。
