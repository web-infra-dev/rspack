---
name: runner-stuck-rescue
description: Rescue a GitHub Actions run that is stuck queued because no self-hosted runner picked up the job, by temporarily swapping the runner-label repo variable to a GitHub-hosted runner, then cancelling and re-running the stuck job. Use when a self-hosted CI job stays queued/waiting for a long time with no runner assigned, when the user shares a stuck workflow run link, or mentions "runner stuck", "no runner", "queued forever", "没机器", "没有 runner".
---

# Runner Stuck Rescue

Unblock a run that is stuck because a self-hosted (`rspack-*`) runner is offline / not
picking up the job, by temporarily routing that job to a GitHub-hosted runner.

> [!WARNING]
> Repo variables are **global**: changing one affects every running and future
> workflow, not just this run. Only do this for a genuine runner outage, keep the
> change as small as possible (one variable), and **restore it as soon as the
> self-hosted runners recover** (Step 6). Consider telling the team first.

Requires `gh` authenticated with **admin / variable-write** access to the repo.

## How runners are wired in this repo (read first)

Every runner label is `fromJSON(vars.<VAR> || '<hosted-fallback>')`. Two facts drive the
whole procedure:

- **The value is a JSON string.** `gh variable list` shows it _with_ quotes, e.g.
  `"rspack-ubuntu-22.04-large"` — the quotes are part of the value (required by
  `fromJSON`). Every set/restore must keep the inner quotes.
- **The `vars.<VAR>` reference lives in one of two places**, so don't expect
  `runs-on: ${{ vars.X }}` on the stuck job itself:
  - **Passed as an input** (most jobs): the caller passes it into a reusable workflow's
    `runner` / `test-runner` input —
    `runner: ${{ vars.LINUX_SELF_HOSTED_RUNNER_LABELS || '"ubuntu-22.04"' }}` — and the
    reusable workflow does `runs-on: ${{ fromJSON(inputs.runner) }}`.
  - **Directly** in a leaf job's `runs-on` (lint / rust-check / size jobs) —
    `runs-on: ${{ fromJSON(vars.CI_LINUX_MINI_RUNNER || '"ubuntu-latest"') }}`.

Only four variables actually gate self-hosted runners (see Reference); the rest are not
referenced by current workflows and swapping them does nothing.

## Inputs

A stuck run URL, e.g. `https://github.com/web-infra-dev/rspack/actions/runs/<run_id>`
(optionally `.../job/<job_id>`). `gh repo view` rejects a deep run URL, so parse the
parts out of the URL directly:

```bash
URL="https://github.com/web-infra-dev/rspack/actions/runs/<run_id>"
REPO=$(sed -E 's#https?://github.com/([^/]+/[^/]+)/.*#\1#' <<<"$URL")   # -> web-infra-dev/rspack
run_id=$(sed -E 's#.*/runs/([0-9]+).*#\1#' <<<"$URL")
```

## Step 1 — Confirm it is stuck on a missing runner

```bash
gh api repos/$REPO/actions/runs/$run_id/jobs --paginate \
  --jq '.jobs[] | select(.status=="queued")
        | {name, status, runner_name, labels, created:.created_at, started:.started_at}'
```

It is a **runner-stuck** case (not a normal wait) only when a job has:
`status = queued`, `runner_name = null`, `started_at = null`, a self-hosted label
(`rspack-*`) in `labels`, and it has been queued a long time. If jobs are
`in_progress`, or the _run_ is waiting on an approval gate, **stop** — this skill
does not apply.

## Step 2 — Find the _exact_ variable feeding the stuck job's runner

1. Read the stuck job's evaluated self-hosted label (the `rspack-*` entry in `.labels`).
2. Map it to a variable — the self-hosted set is small:

   | Stuck self-hosted label     | Variable                            |
   | --------------------------- | ----------------------------------- |
   | `rspack-ubuntu-22.04-large` | `LINUX_SELF_HOSTED_RUNNER_LABELS`   |
   | `rspack-ubuntu-22.04-mini`  | `CI_LINUX_MINI_RUNNER`              |
   | `rspack-windows-2022-large` | `WINDOWS_SELF_HOSTED_RUNNER_LABELS` |
   | `rspack-darwin-14-medium`   | `MAC_SELF_HOSTED_RUNNER_LABELS`     |

3. To pin it for a specific job, trace the source. Get the workflow file, then find the
   job — or the `uses:` job that calls a reusable workflow and passes `runner:` /
   `test-runner:` — and read the `vars.<VAR>` there:

```bash
gh api repos/$REPO/actions/runs/$run_id --jq .path      # -> .github/workflows/<file>.yml
grep -rnE "SELF_HOSTED_RUNNER_LABELS|CI_LINUX_MINI_RUNNER" .github/workflows/
```

Cross-check by value (values are JSON-quoted, so parse before matching):

```bash
gh variable list --json name,value \
  | jq -r --arg L "<stuck-label>" '.[] | select((.value|(fromjson? // .)|tostring)|contains($L)) | .name'
```

Confirm the chosen `VAR` with the user before changing it.

## Step 3 — Back up all variables

```bash
gh variable list --json name,value > runner-vars-backup.json   # keep this file (values keep their quotes)
gh variable list                                                # human-readable copy
```

Cross-check against the hardcoded reference in this file so a lost backup can still be
restored.

## Step 4 — Swap the one variable to a GitHub-hosted runner (keep the JSON quotes)

The replacement must be valid JSON, so **keep the inner quotes**:

```bash
gh variable set <VAR> --body '"ubuntu-latest"'    # linux  (mini or large)
gh variable set <VAR> --body '"windows-latest"'   # windows
gh variable set <VAR> --body '"macos-latest"'     # macos
gh variable get <VAR>                              # verify -> value still includes the quotes
```

A bare `--body ubuntu-latest` (no inner quotes) breaks `fromJSON` and every job on that
variable fails to start. Alternative: `gh variable delete <VAR>` also routes to hosted,
because every reference falls back to `|| '"<hosted>"'` when unset — do this only with
the Step 3 backup in hand.

## Step 5 — Cancel the stuck run, then re-run it

A variable is only read when a job **starts**, so the run must be cancelled and restarted
to pick up the new label.

```bash
gh run cancel $run_id
gh run watch $run_id --exit-status || true     # block until the cancel settles (non-zero on cancelled is expected)
gh run rerun $run_id                           # re-run the whole (now-cancelled) run with the new label
gh run watch $run_id                           # follow; report the final result (conclusion + URL) back to the requester
```

`gh run rerun --failed` is leaner (skips jobs that already succeeded) but relies on
cancelled jobs counting as "failed" — if unsure, use the full `gh run rerun` above.

If the rerun **still** stays queued, an upstream dependency job (e.g. a `build` the test
depends on) is also on a self-hosted runner. Repeat Steps 2–4 for its variable, rerun.

## Step 6 — Restore when runners recover (do not skip)

The backup preserves the exact JSON-quoted values; single-quote them on restore:

```bash
jq -r '.[] | "gh variable set \(.name) --body '\''\(.value)'\''"' runner-vars-backup.json
# run the printed line(s) for the variable(s) you changed, then verify:
gh variable get <VAR>
```

## Reference

**Self-hosted detection:** a runner is self-hosted iff its (unquoted) value starts with
`rspack-`. Hosted values (`ubuntu-latest`, `ubuntu-22.04`, `windows-latest`,
`macos-latest`, `blacksmith-*`) are never runner-stuck for this reason.

**OS → hosted replacement** (JSON-quoted in the `gh variable set` body):

| Label contains     | OS      | `--body`             |
| ------------------ | ------- | -------------------- |
| `ubuntu` / `linux` | Linux   | `'"ubuntu-latest"'`  |
| `windows` / `win`  | Windows | `'"windows-latest"'` |
| `darwin` / `mac`   | macOS   | `'"macos-latest"'`   |

**Original values (JSON strings — restore exactly, quotes included):**

| Variable                            | Original value (JSON)         | Gates a self-hosted job?       |
| ----------------------------------- | ----------------------------- | ------------------------------ |
| `LINUX_SELF_HOSTED_RUNNER_LABELS`   | `"rspack-ubuntu-22.04-large"` | yes — runner input + direct    |
| `CI_LINUX_MINI_RUNNER`              | `"rspack-ubuntu-22.04-mini"`  | yes — lint/rust/size `runs-on` |
| `WINDOWS_SELF_HOSTED_RUNNER_LABELS` | `"rspack-windows-2022-large"` | yes — runner input             |
| `MAC_SELF_HOSTED_RUNNER_LABELS`     | `"rspack-darwin-14-medium"`   | yes — runner input             |
| `CI_MACOS_BUILD_RUNNER`             | `"rspack-darwin-14-medium"`   | no — not referenced (grep)     |
| `CI_MACOS_TEST_RUNNER`              | `"rspack-darwin-14-medium"`   | no — not referenced (grep)     |
| `CI_WINDOWS_BUILD_RUNNER`           | `"rspack-windows-2022-large"` | no — not referenced (grep)     |
| `CI_WINDOWS_TEST_RUNNER`            | `"windows-latest"`            | no — not referenced (grep)     |

Only the four "yes" variables can cause a runner-stuck job. The four `CI_*_BUILD/TEST_RUNNER`
entries are not referenced in current `.github/workflows/` (verified by grep), so swapping
them has no effect.
