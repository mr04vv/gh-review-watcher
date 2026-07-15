#!/usr/bin/env bash
set -euo pipefail

# gh-review-watcher on_remove hook: リストから消えたPRのレビュータブを閉じる
# Arguments: {number} {repo}
NUMBER="$1"
REPO="$2"

TAB_NAME="Review: ${REPO}#${NUMBER}"

# レビュータブが存在しなければ何もしない
TAB_ID=$(zellij action list-tabs --json 2>/dev/null \
  | jq -r --arg name "$TAB_NAME" '.[] | select(.name == $name) | .tab_id' 2>/dev/null)

if [[ -z "$TAB_ID" ]]; then
  exit 0
fi

zellij action close-tab-by-id "$TAB_ID"
echo "[CLOSED TAB] ${TAB_NAME}" >> /tmp/gh-review-watcher-hooks.log
