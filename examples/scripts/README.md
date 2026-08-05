# examples/scripts

`examples/config.toml` の hook から呼ばれるヘルパースクリプトの例です。
そのまま使う必要はなく、自分のワークフローに合わせて改変する土台としてどうぞ。

## スクリプト

| スクリプト | 呼ばれる hook | 役割 |
|---|---|---|
| `open-review-tab.sh` | `on_new_pr` (review-tab) / `actions` | レビュー用の新タブを開いて `review-pr` を起動。**実行時に herdr / zellij を判定する**ので、どちらのマルチプレクサ内で watcher を動かしても同じ config で機能する。herdr ではタブ作成に `--no-focus` を使い、`review-pr` 終了時にタブを自動で閉じる |
| `review-pr.sh` | `open-review-tab.sh` 経由 | タブ内で PR をレビュー。**PR の CI 状態を取得してレポートに表示し、CI が失敗していれば自動で `request-changes` を送る**（二重送信ガード付き）。`claude -p /review` で分析し固定テンプレに整形して表示、`[a]pprove / [c]omment / [d]iscuss / [o]pen / [q]uit` を対話選択 |
| `close-merged-review-tab.sh` | `on_remove` | マージ/クローズ等で消えた PR のレビュータブを閉じる（herdr / zellij 両対応） |

## 設置

`config.toml` は拡張子なしのコマンド名 (`open-review-tab` / `review-pr` /
`close-merged-review-tab`) で参照しているので、PATH 上にその名前で置く
（シンボリックリンク推奨）:

```sh
mkdir -p ~/.local/bin
ln -sf "$PWD/open-review-tab.sh"         ~/.local/bin/open-review-tab
ln -sf "$PWD/review-pr.sh"               ~/.local/bin/review-pr
ln -sf "$PWD/close-merged-review-tab.sh" ~/.local/bin/close-merged-review-tab
# ~/.local/bin が PATH に入っていること
```

## 依存

- [Zellij](https://zellij.dev/) または [herdr](https://herdr.dev/) — タブ操作。
  どちらを使うかはスクリプトが実行時の環境変数 (`ZELLIJ` / `HERDR_ENV`) で判定する
- [Claude Code](https://claude.com/claude-code) — `claude` CLI。`review-pr.sh` は
  スラッシュコマンド `/review`、`config.toml` の yolo-review hook は `/yolo-review`
  を使う（`/yolo-review` は各自で用意する Claude Code skill / command）
- `gh` (GitHub CLI), `jq`

いずれも `{repo}` `{number}` `{url}` などの値は gh-review-watcher が hook 実行時に
展開して引数で渡す。スクリプト自体にトークン等の秘密は含まれていない。
