#!/usr/bin/env bash
set -euo pipefail

# gh-review-watcher on_remove hook: リストから消えたPRのレビュータブを閉じる。
# open-review-tab.sh と同様、herdr / zellij を実行時に判定する。
# Arguments: {number} {repo}
NUMBER="$1"
REPO="$2"

TAB_NAME="Review: ${REPO}#${NUMBER}"

if [[ "${HERDR_ENV:-}" == "1" ]]; then
  TAB_ID=$(herdr tab list 2>/dev/null \
    | jq -r --arg name "$TAB_NAME" \
        '.result.tabs[] | select(.label == $name) | .tab_id' 2>/dev/null \
    | head -1)

  if [[ -z "$TAB_ID" ]]; then
    exit 0
  fi

  herdr tab close "$TAB_ID" >/dev/null
elif [[ -n "${ZELLIJ:-}" ]]; then
  # レビュータブが存在しなければ何もしない
  TAB_ID=$(zellij action list-tabs --json 2>/dev/null \
    | jq -r --arg name "$TAB_NAME" '.[] | select(.name == $name) | .tab_id' 2>/dev/null)

  if [[ -z "$TAB_ID" ]]; then
    exit 0
  fi

  zellij action close-tab-by-id "$TAB_ID"
else
  exit 0
fi

echo "[CLOSED TAB] ${TAB_NAME}" >> /tmp/gh-review-watcher-hooks.log
