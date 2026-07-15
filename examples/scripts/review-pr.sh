#!/usr/bin/env bash
set -euo pipefail

URL="$1"
NUMBER="$2"
REPO="$3"

echo "🔍 Reviewing PR #${NUMBER} in ${REPO}..."
echo ""

# Step 0: PR のメタデータを gh で取得 (Author, Title, Base/Head, Additions/Deletions など)
PR_META_JSON=$(gh pr view "$NUMBER" -R "$REPO" --json \
  title,author,baseRefName,headRefName,additions,deletions,changedFiles,isDraft,mergeStateStatus,createdAt 2>/dev/null || echo "{}")

PR_TITLE=$(echo "$PR_META_JSON" | jq -r '.title // "(タイトル取得失敗)"')
PR_AUTHOR=$(echo "$PR_META_JSON" | jq -r '.author.login // "?"')
PR_BASE=$(echo "$PR_META_JSON" | jq -r '.baseRefName // "?"')
PR_HEAD=$(echo "$PR_META_JSON" | jq -r '.headRefName // "?"')
PR_ADD=$(echo "$PR_META_JSON" | jq -r '.additions // 0')
PR_DEL=$(echo "$PR_META_JSON" | jq -r '.deletions // 0')
PR_FILES=$(echo "$PR_META_JSON" | jq -r '.changedFiles // 0')
PR_DRAFT=$(echo "$PR_META_JSON" | jq -r '.isDraft // false')
PR_STATE=$(echo "$PR_META_JSON" | jq -r '.mergeStateStatus // "?"')
PR_CREATED=$(echo "$PR_META_JSON" | jq -r '.createdAt // "?"' | cut -d'T' -f1)
PR_DRAFT_BADGE=""
[ "$PR_DRAFT" = "true" ] && PR_DRAFT_BADGE=" [DRAFT]"

# Step 1: claude -p でレビュー実行、結果を $REVIEW_RESULT に保存 (生データは後段の [c] でも使う)
REVIEW_RESULT=$(claude --dangerously-skip-permissions -p "/review ${URL}" 2>&1)

# Step 2: $REVIEW_RESULT を固定テンプレートに再整形 (タブで一貫した5セクション構造で見るため)
REFORMAT_PROMPT="以下は PR #${NUMBER} (${REPO}) に対するコードレビュー結果です。
これを下記の固定テンプレートに再整形してください。
ターミナル (zellij タブ) で人間が読むことを想定しており、emoji と区切り罫線で
セクションを視認しやすくします。 内容は元レビューにあるものだけを使い、勝手に増やさない。

== 出力テンプレート (このまま、コードフェンスで包まない) ==

╔══════════════════════════════════════════════════════════════════╗
║  PR #${NUMBER} — ${REPO}${PR_DRAFT_BADGE}
║  ${PR_TITLE}
╚══════════════════════════════════════════════════════════════════╝

## 📌 PR Info
- 👤 Author:  ${PR_AUTHOR}
- 🌿 Branch:  \`${PR_HEAD}\` → \`${PR_BASE}\`
- 📊 Diff:    +${PR_ADD} / -${PR_DEL}  (${PR_FILES} files)
- 🗓  Created: ${PR_CREATED}
- 🧭 State:   ${PR_STATE}
- 🔗 URL:     ${URL}

────────────────────────────────────────────────────────────────────

## 🎯 Verdict
<APPROVE | REQUEST_CHANGES | DISCUSS | SKIP> — <1行の理由>

## 📋 Summary
- <変更の要点を 3 bullet 以内>

────────────────────────────────────────────────────────────────────

## ⛔ Blockers (N)
- [ ] <file:line> — <merge をブロックすべき問題>
  <なぜブロッカーか / 想定影響を 1-2 行で>

(無ければ \"なし\" の一行のみ)

────────────────────────────────────────────────────────────────────

## 💡 Suggestions (N)
- <file:line> — <推奨修正>
  <推奨理由を 1-2 行で>

(無ければ \"なし\" の一行のみ)

────────────────────────────────────────────────────────────────────

## 📝 Notes (N)
- <file:line> — <それ以外の気づき / Praise / 確認したい点>

(無ければ \"なし\" の一行のみ)

== 厳格なルール ==

- PR Info セクション (👤 Author / 🌿 Branch / 📊 Diff / 🗓 Created / 🧭 State / 🔗 URL) はテンプレートに記載された値をそのまま出力する。 値の改変・省略・追加禁止。
- 元レビューに無い事実を追加しない。 再整形と要約のみ。
- 各セクションの (N) は実件数を入れる (例: \"## ⛔ Blockers (2)\")。 0 なら (0)。
- 元レビューが \"issues なし\" / \"No issues found\" 系なら Verdict=APPROVE。 Blockers/Suggestions/Notes は (0) で \"なし\"。
- 元レビューが \"PR が closed/draft でレビュー対象外\" 系で実質レビューされていない場合は Verdict=SKIP、各セクションは \"レビュー対象外\"。
- Verdict の絵文字対応: 🎯 は常にそのまま、Verdict 文字列の後に APPROVE なら ✅ / REQUEST_CHANGES なら ⛔ / DISCUSS なら 💬 / SKIP なら ⏭️ を付ける。 例: \"✅ APPROVE — テストカバー十分\"
- セクション順序・見出し・絵文字・罫線は固定。 空でも見出しは消さない。
- Markdown コードフェンスで全体を包まない (タブに垂れ流すため)。
- テンプレート見出し (Verdict/Summary/Blockers/Suggestions/Notes) は英語固定、内容は元レビューの言語に従う。
- 元レビューに具体的な file:line が含まれていれば必ず残す (情報量を落とさない)。

== 再整形対象 (元レビュー) ==

${REVIEW_RESULT}"

FORMATTED_RESULT=$(claude --dangerously-skip-permissions -p "$REFORMAT_PROMPT" 2>&1 || echo "")

# reformat が空 / 失敗したら fallback として元レビューをそのまま出す
if [[ -z "${FORMATTED_RESULT// }" ]]; then
  echo "⚠️  reformat 失敗。 元レビューをそのまま表示します。"
  echo ""
  echo "$REVIEW_RESULT"
else
  echo "$FORMATTED_RESULT"
fi
echo ""

# 分析完了後、このレビュータブにフォーカスを移動
zellij action go-to-tab-name "Review: ${REPO}#${NUMBER}" 2>/dev/null || true

# Step 2: 選択肢を提示
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  [a] Approve this PR"
echo "  [c] Comment concerns as pending review (open in browser)"
echo "  [d] Discuss with Claude Code"
echo "  [o] Open in browser"
echo "  [q] Quit"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
read -r -p "Choose action: " choice

case "$choice" in
  a)
    gh pr review "$NUMBER" -R "$REPO" --approve --body "LGTM 👍 (Reviewed by Claude Code)"
    echo "✅ Approved!"
    ;;
  c)
    echo "🤖 Extracting concerns as inline comments..."
    OWNER="${REPO%/*}"
    REPO_NAME="${REPO#*/}"
    COMMIT_ID=$(gh pr view "$NUMBER" -R "$REPO" --json headRefOid -q '.headRefOid')
    DIFF=$(gh pr diff "$NUMBER" -R "$REPO")

    # diffをパースして各行に絶対行番号を付与（Claudeが行番号を計算する必要をなくす）
    # +行とコンテキスト行（スペース始まり）の両方にアノテーションを付ける
    # GitHub APIはdiff内のコンテキスト行にもコメントできるため
    ANNOTATED_DIFF=$(echo "$DIFF" | awk '
      /^diff --git/ { file="" }
      /^\+\+\+ / { file=substr($0, 7); print; next }
      /^--- / { print; next }
      /^@@ / {
        s = $0
        sub(/^@@ -[0-9,]+ \+/, "", s)
        sub(/,.*/, "", s)
        newline = s + 0
        print
        next
      }
      file != "" && /^[-+ ]/ {
        prefix = substr($0, 1, 1)
        if (prefix == "-") {
          print
        } else if (prefix == "+") {
          printf "[LINE %d] %s\n", newline, $0
          newline++
        } else {
          printf "[LINE %d] %s\n", newline, $0
          newline++
        }
        next
      }
      { print }
    ')

    COMMENTS_JSON=$(claude --dangerously-skip-permissions -p "以下はPRの annotated diff と事前レビュー結果です。レビュー結果に挙がっている懸念事項を、該当する行への inline review comment として JSON 配列で出力してください。

出力形式（この形式のJSON配列のみ、余計なテキストなし、コードフェンスなし）:
[{\"path\": \"src/foo.ts\", \"line\": 42, \"side\": \"RIGHT\", \"body\": \"懸念内容\"}]

- path: 変更されたファイルのパス（+++ b/... のパス。先頭の b/ は除く）
- line: [LINE N] タグに記載された番号をそのまま使うこと。自分で計算しないでください。
- side: 常に \"RIGHT\"
- body: 懸念事項の内容（日本語で簡潔に）
- 懸念事項がなければ空配列 []

重要: 各追加行(+)とコンテキスト行(スペース始まり)の先頭に [LINE N] タグが付いています。この N が変更後ファイルの絶対行番号です。必ずこの番号をそのまま使ってください。削除行(-)にはタグがありません。

--- ANNOTATED DIFF ---
${ANNOTATED_DIFF}

--- REVIEW ---
${REVIEW_RESULT}")

    # 余計な装飾を除去
    COMMENTS_JSON=$(echo "$COMMENTS_JSON" | sed -n '/^\[/,/^\]/p')

    if [[ -z "$COMMENTS_JSON" || "$COMMENTS_JSON" == "[]" ]]; then
      echo "ℹ️  懸念事項は抽出されませんでした。"
      exit 0
    fi

    echo "$COMMENTS_JSON" | jq .

    # pending review を作成（event を指定しないと pending になる）
    PAYLOAD=$(jq -n --arg commit "$COMMIT_ID" --argjson comments "$COMMENTS_JSON" \
      '{commit_id: $commit, comments: $comments}')

    echo "$PAYLOAD" | gh api "repos/${OWNER}/${REPO_NAME}/pulls/${NUMBER}/reviews" \
      --method POST --input - > /dev/null

    echo "✅ Pending review created. Opening browser..."
    open "${URL}/files"
    ;;
  d)
    exec claude --dangerously-skip-permissions \
      "PR #${NUMBER} (${REPO}) について議論しましょう。URL: ${URL}

以下はClaude Codeによる事前レビュー結果です:

${REVIEW_RESULT}"
    ;;
  o)
    open "$URL"
    ;;
  q)
    exit 0
    ;;
esac
