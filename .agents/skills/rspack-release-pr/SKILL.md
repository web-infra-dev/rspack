---
name: rspack-release-pr
description: Create the official Rspack release pull request for a stable or pre-release version bump. Use when the task is to prepare a formal release branch from a clean checkout, sync to the latest origin/main, run `./x version` with an optional `--pre alpha|beta|rc`, confirm the resulting JavaScript and Rust versions with the user, open the release PR, trigger Ecosystem CI, and report the PR plus workflow URLs.
---

# Rspack Release PR

## Overview

Create the formal release pull request from a clean checkout, stop for explicit version confirmation, then publish the branch and trigger the follow-up workflow.

Prefer this workflow for official `major`, `minor`, or `patch` releases, including `alpha`, `beta`, and `rc` pre-releases. Do not use it for snapshot, debug, or canary flows.

## Defaults

- Default bump version: `patch`
- Allowed bump versions: `major`, `minor`, `patch`
- Allowed pre tags: `alpha`, `beta`, `rc`
- Branch name: `chore/release-YYYY-MM-DD`
- Commit message: `chore(release): release <new_js_version>`
- PR title: `chore: release version <new_js_version>`

## Safety gates

- Treat this workflow as destructive to unstaged local changes.
- If `git status --short` is not clean and the user did not explicitly ask to discard local unstaged changes in this turn, stop and confirm before cleaning.
- If staged changes already exist, stop and ask how to handle them. A release PR should start from a fully clean tree.
- After `./x version ...`, stop and show both versions. Do not commit, push, open the PR, or trigger workflows until the user confirms the versions.

## Workflow

### 1) Clean the worktree

Inspect the tree first:

```sh
git status --short
git diff --cached --quiet
```

If there are staged changes, stop.

If there are only unstaged tracked or untracked changes and the user explicitly authorized discarding them, clean them non-interactively:

```sh
git restore --worktree -- .
git clean -fd
git status --short
```

Proceed only when `git status --short` is empty.

### 2) Sync to the latest `origin/main`

```sh
git fetch origin --prune
git switch main 2>/dev/null || git switch -c main --track origin/main
git reset --hard origin/main
git status --short
```

The release branch should start from the exact current `origin/main`, not from an older local branch tip.

### 3) Reinstall dependencies

```sh
pnpm install
```

Run this before the version bump so the local release tooling and workspace dependencies are fresh.

### 4) Create the release branch

Use the current local date in `YYYY-MM-DD` format:

```sh
RELEASE_DATE="$(date +%F)"
BRANCH="chore/release-${RELEASE_DATE}"
git switch -c "$BRANCH"
```

Reuse the same `RELEASE_DATE` later in the PR body.

### 5) Bump the version

Choose the bump version from the user request:

- `major`, `minor`, or `patch`
- Default to `patch` when the user does not specify one
- If the user explicitly asks for `alpha`, `beta`, or `rc`, add `--pre <tag>`

Examples:

```sh
./x version patch
./x version minor
./x version patch --pre rc
./x version minor --pre beta
```

`./x version ...` already updates the published JavaScript package versions, updates the Rust workspace version through the release tooling, runs `cargo codegen`, and runs `pnpm run format:js`.

After the command finishes, read the authoritative versions from:

- JavaScript packages: `packages/rspack/package.json`
- Rust crates: `[workspace.package].version` in the root `Cargo.toml`

Example commands:

```sh
JS_VERSION="$(pnpm exec node -e 'const fs=require("node:fs"); console.log(JSON.parse(fs.readFileSync("packages/rspack/package.json","utf8")).version)')"
RUST_VERSION="$(pnpm --dir scripts exec node -e 'const fs=require("node:fs"); const TOML=require("@iarna/toml"); console.log(TOML.parse(fs.readFileSync("../Cargo.toml","utf8")).workspace.package.version)')"
printf 'JavaScript packages version: %s\nRust crates version: %s\n' "$JS_VERSION" "$RUST_VERSION"
```

Show both values to the user and wait for confirmation before continuing.

### 6) Commit, push, and open the PR

Once the user confirms the versions, prepare the release metadata:

```sh
RELEASE_OWNER="$(gh api user --jq .login 2>/dev/null || git config user.name)"
```

If `RELEASE_OWNER` is empty, ask the user for the value before creating the PR.

Stage and commit the version bump:

```sh
git add -A
git commit -m "chore(release): release ${JS_VERSION}"
git push -u origin "$BRANCH"
```

Create an English PR body with the required fields:

```md
## Release Information

- Released By: <release_owner>
- Release Date: <release_date>
- JavaScript Packages Version: <js_version>
- Rust Crates Version: <rust_version>
```

Then open the PR:

```sh
cat >/tmp/rspack-release-pr-body.md <<EOF
## Release Information

- Released By: ${RELEASE_OWNER}
- Release Date: ${RELEASE_DATE}
- JavaScript Packages Version: ${JS_VERSION}
- Rust Crates Version: ${RUST_VERSION}
EOF

gh pr create \
  --base main \
  --head "$BRANCH" \
  --title "chore: release version ${JS_VERSION}" \
  --body-file /tmp/rspack-release-pr-body.md
```

Capture the PR number and URL immediately after creation:

```sh
PR_NUMBER="$(gh pr view --json number --jq '.number')"
PR_URL="$(gh pr view --json url --jq '.url')"
```

### 7) Trigger `ecosystem-ci`

The workflow file is [`.github/workflows/ecosystem-ci.yml`](../../../.github/workflows/ecosystem-ci.yml) and the workflow display name is `Ecosystem CI`.

Trigger it with the PR number:

```sh
DISPATCH_USER="$(gh api user --jq .login)"
PREV_ECOSYSTEM_CI_RUN_ID="$(gh run list \
  --workflow ecosystem-ci.yml \
  --event workflow_dispatch \
  --user "$DISPATCH_USER" \
  --limit 1 \
  --json databaseId \
  --jq '.[0].databaseId // ""')"

gh workflow run ecosystem-ci.yml --ref main \
  -f pr="$PR_NUMBER" \
  -f suite=- \
  -f suiteRefType=precoded \
  -f suiteRef=precoded
```

Then wait a few seconds and poll until a new workflow-dispatch run appears for this workflow:

```sh
sleep 5
while :; do
  RUN_ID="$(gh run list \
    --workflow ecosystem-ci.yml \
    --event workflow_dispatch \
    --user "$DISPATCH_USER" \
    --limit 1 \
    --json databaseId \
    --jq '.[0].databaseId // ""')"
  ECOSYSTEM_CI_URL="$(gh run list \
    --workflow ecosystem-ci.yml \
    --event workflow_dispatch \
    --user "$DISPATCH_USER" \
    --limit 1 \
    --json url \
    --jq '.[0].url // ""')"

  if [ -n "$RUN_ID" ] && [ "$RUN_ID" != "$PREV_ECOSYSTEM_CI_RUN_ID" ]; then
    break
  fi

  sleep 5
done
```

Do not report the previous latest run. Only report the first new `workflow_dispatch` run from `DISPATCH_USER` that appears after your dispatch for `PR_NUMBER`.

### 8) Report the result

End by telling the user:

- `PR URL: <pr_url>`
- `Ecosystem CI URL: <ecosystem_ci_url>`

Include the confirmed JavaScript and Rust version numbers again if that helps the handoff.

## Common traps

- Do not keep local staged work mixed into the release PR.
- Do not skip the user confirmation step after `./x version ...`.
- Do not derive the PR title from template packages under `packages/create-rspack/template-*`; use the published version in `packages/rspack/package.json`.
- Do not forget that the Rust version is authoritative in the root `Cargo.toml`, even though `./x version` updates many files.
- Do not stop at the workflow dispatch command. Always retrieve the PR URL and the workflow run URL before reporting completion.
