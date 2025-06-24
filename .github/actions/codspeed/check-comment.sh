#!/bin/bash
set -euo pipefail

PR_NUMBER="${PR_NUMBER:-}"
REPO="${GITHUB_REPOSITORY}"
HEAD_SHA="${HEAD_SHA:-}"
GITHUB_API_URL="${GITHUB_API_URL:-https://api.github.com}"
AUTH_HEADER="Authorization: Bearer ${GITHUB_TOKEN}"
POLL_INTERVAL=5
MAX_RETRIES=600  # 5 minutes

if [[ -z "$PR_NUMBER" || -z "$HEAD_SHA" || -z "$GITHUB_TOKEN" ]]; then
  echo "Missing required environment variables: PR_NUMBER, HEAD_SHA, or GITHUB_TOKEN"
  exit 1
fi

HEAD_SHORT="${HEAD_SHA:0:7}"
COMMENTS_URL="$GITHUB_API_URL/repos/$REPO/issues/$PR_NUMBER/comments"

echo "Polling for codspeed-hq[bot] comment matching SHA ($HEAD_SHORT) on PR #$PR_NUMBER..."

for ((i=1; i<=MAX_RETRIES; i++)); do
  COMMENTS=$(curl -s -H "$AUTH_HEADER" "$COMMENTS_URL")

  MATCHING_COMMENT=$(echo "$COMMENTS" | jq -r --arg sha "$HEAD_SHORT" '
    .[] |
    select(.user.login == "codspeed-hq[bot]") |
    select(.body | contains($sha)) |
    .body
  ')

  if [[ -n "$MATCHING_COMMENT" ]]; then
    echo "Found matching comment:"
    echo "----------------------------------------"
    echo "$MATCHING_COMMENT"
    echo "----------------------------------------"
    echo ""

    if echo "$MATCHING_COMMENT" | grep -q "degrade performances by"; then
      echo "❌ Performance regression detected. Failing..."
      exit 1
    else
      echo "✅ No performance degradation. Success."
      exit 0
    fi
  fi

  echo "⌛ Waiting... ($i/$MAX_RETRIES)"
  sleep "$POLL_INTERVAL"
done

echo "❌ Timeout: No matching codspeed comment found with SHA ($HEAD_SHORT)"
exit 1
