#!/usr/bin/env bash
set -euo pipefail

# gh-review-watcher on_new_pr hook: PRレビュータブを開く。
# watcher が動いているマルチプレクサ (herdr / zellij) を実行時に判定するので、
# 同じ config.toml がどちらのセッションでも壊れない。
# Arguments: {url} {number} {repo}

URL="$1"
NUMBER="$2"
REPO="$3"

TAB_NAME="Review: ${REPO}#${NUMBER}"

if [[ "${HERDR_ENV:-}" == "1" && -n "${HERDR_WORKSPACE_ID:-}" ]]; then
  # herdr には --close-on-exit が無い: review-pr の終了後に自分のタブを閉じる
  # コマンドを連結して同じ挙動にする ([q] やエラー終了も含む)。
  # PRがレビュー中にマージされた場合は on_remove hook が先に閉じる。
  # --no-focus があるので zellij 側で必要な go-to-previous-tab は不要。
  OUT=$(herdr tab create --workspace "$HERDR_WORKSPACE_ID" --label "$TAB_NAME" --no-focus)
  PANE_ID=$(printf '%s' "$OUT" | jq -r '.result.root_pane.pane_id')
  TAB_ID=$(printf '%s' "$OUT" | jq -r '.result.tab.tab_id')

  # pane run は引数をスペース結合してペインのシェルに打ち込むため、
  # 引数境界はここでシングルクォートを埋め込んで保つ。
  herdr pane run "$PANE_ID" \
    "review-pr '${URL}' '${NUMBER}' '${REPO}'; herdr tab close '${TAB_ID}'" >/dev/null
elif [[ -n "${ZELLIJ:-}" ]]; then
  # 分析完了後にタブへフォーカスが移るため、起動直後は元タブに戻す
  zellij action new-tab --name "$TAB_NAME" --close-on-exit -- \
    review-pr "$URL" "$NUMBER" "$REPO"
  zellij action go-to-previous-tab
else
  echo "[open-review-tab] no multiplexer detected (HERDR_ENV/ZELLIJ unset); skipping" >&2
fi
