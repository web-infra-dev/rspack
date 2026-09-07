---
name: rstack-eco-ci-debug
description: Debug Rstack ecosystem CI failures and attribute the real source PR or downstream change. Always use this skill when the user mentions Rspack eco-ci, rstack-ecosystem-ci, a suite turning red, a downstream regression, a green-to-red pivot, canary bisect, @rspack-canary/core, or daily eco-ci triage — even if they only ask "why is this suite failing", "which PR broke it", or "is this Rspack's fault". Use it to avoid over-blaming the first Rspack commit that appears red in status data.
metadata:
  internal: true
---

# Rstack Eco CI Debug

Use this skill to debug Rstack ecosystem CI failures without over-blaming the first Rspack commit that appears red in status data.

This version covers the Rspack stack.

## Preconditions

Before starting, ask the user which local checkout paths they have available. Do not assume machine-specific paths.

- **Local Rspack checkout** — required for inspecting commits, resolving canary SHAs, and reviewing PR diffs. Ask for it before running any `git -C <rspack-path>` command.
- **Local downstream project checkout** — required when using `pnpm.overrides` to test specific Rspack versions (for example, during canary bisect). Ask for it before making approved temporary reproduction edits to `package.json`, `pnpm-lock.yaml`, or equivalent package-manager files.
- **Local `rstack-ecosystem-ci` checkout** — optional. If available, use its `data/rspack.json` as the first local status source. Pass the checkout path to local helpers that read git refs, for example `scripts/rspack-status.sh --repo <ecosystem-ci-path>`.
- **GitHub access** — prefer authenticated `gh` for `web-infra-dev/rspack` and `rstackjs/rstack-ecosystem-ci`. If `gh` is unavailable, use `origin/data:rspack.json` from a local `rstack-ecosystem-ci` checkout, workflow/job URLs, and the GitHub connector or public pages where available; state any evidence gap in the report.

Fetch the local Rspack repo before resolving commits:

```bash
git -C <rspack-path> fetch origin main --tags
```

- Treat GitHub Actions job logs as the source of truth for failure signatures.
- Do not modify project files unless the user explicitly asks for a fix or approves temporary reproduction edits. Temporary reproduction edits must be recorded and restored before reporting results unless the user asks to keep them.

## Investigation Model

Rspack eco-ci runs a downstream project matrix against a freshly built Rspack artifact. A suite turning red means that a specific combination failed:

```text
current downstream project state + tested Rspack artifact
```

It does not automatically mean the visible Rspack pivot PR is the true root cause. Downstream dependency updates, snapshot changes, test logic changes, and Rspack release/canary windows can all create misleading pivots.

Some eco-ci failures are flaky even when they appear as a clean green-to-red transition. Repeated selector timeouts, browser navigation waits, dev-server readiness failures, and network-idle waits must be checked against old commit comments, old failed runs, and automation memory before attributing them to the current pivot.

Always distinguish:

- `Surface attribution`: the Rspack commit/PR where status data first shows the suite red.
- `Actual source`: the PR, version window, or downstream change that actually introduced the failing condition.
- `Failure signature`: the stable error text, command, assertion diff, stack, or log block used to compare runs.

## Optional Tools

Read the linked reference before using any of these tools. Do not ask the user generically "which tool do you want"; instead, suggest the specific tool that matches the situation. Only invoke a tool when its strict trigger conditions are met; do not run it "just in case".

- **Rspack status helper** — use `scripts/rspack-status.sh --repo <ecosystem-ci-path>` when you need the latest and previous rows from `data/rspack.json`, including failed suite names, job URLs, and suite-set delta. Pass a JSON file path instead when you already have a local snapshot. This helper only summarizes status data; it does not inspect logs or attribute root causes.

- **Canary date bisect** — use in Phase 1 only when the Rspack commit window is too coarse to attribute a PR and downstream causes have already been ruled out. Trigger this when **all** of the following are true:
  - The green-to-red pivot spans **more than 3 Rspack commits** or crosses a release/canary boundary.
  - The failure signature is stable across the red rows in that window.
  - The same Rspack commit does **not** appear in both green and red runs (which would indicate a downstream cause).
    Do **not** trigger when the pivot is a single commit or when the surface PR diff already explains the signature.
    Read [references/canary-date-bisect.md](references/canary-date-bisect.md) and ask the user for the local downstream checkout path and the narrowest failing command.

- **Rsbuild config debug** — use in Phase 1 or Phase 2 only when deciding whether the failing case is related to the current PR requires generated Rsbuild/Rspack config evidence. Trigger it when the user mentions `DEBUG=rsbuild`, asks whether a config is active, or the candidate PR changes behavior controlled by an option, plugin, loader, target, devtool, SSR setting, cache mode, or other config-gated path. Do **not** run it just because the suite is Rsbuild-based. Read [references/rsbuild-config-debug.md](references/rsbuild-config-debug.md) before using it.

- **Automation daily triage** — use instead of the local quick path when the request is a recurring/daily automation, asks for today's/latest Rspack eco-ci status, provides an automation id/memory, or requires delivery to a user/chat. Read [references/automation-daily-triage.md](references/automation-daily-triage.md) before inspecting runs.

- **Deep PR debug** — use in Phase 2 only after a specific Rspack source PR or version window has been identified and the user wants the technical reason behind the failure. Trigger this when **all** of the following are true:
  - The user asks "why did this PR break it", "what is the mechanism", or "how should we fix it".
  - The actual source is a Rspack PR or Rspack version window (not a downstream test change, snapshot update, or dependency bump).
  - Phase 1 has already produced evidence linking the PR to the failure signature.
    Do **not** run deep PR debug on downstream PRs; in those cases, Phase 1 output plus a short note about the downstream change is enough.
    Read [references/deep-pr-debug.md](references/deep-pr-debug.md) automatically once a candidate PR is accepted for deep inspection.

- **PR report comment** — use only after strict attribution identifies a merged source PR as the cause and the user wants to notify the PR author. The source PR can be in Rspack or in the downstream project under test. Trigger this only when:
  - The failure is confidently attributed to a merged source PR (not just a surface pivot).
  - For Rspack attributions, downstream changes, dependency bumps, release windows, and flaky signals have been ruled out.
  - For downstream attributions, the exact downstream PR is the actual source and the visible Rspack pivot has been ruled out.
  - The user gives explicit approval to post to GitHub.
    Read [references/pr-report-comment.md](references/pr-report-comment.md) and prepare a draft comment first; do not post without approval.

## Two-Phase Debug Workflow

Eco-ci debugging has two phases. Do not mix them up.

### Local Triage Quick Path

Use this path for local/manual runs, such as a provided workflow run/job, PR, commit window, or specific suite. For daily automation or latest-status monitoring, read [references/automation-daily-triage.md](references/automation-daily-triage.md) instead.

1. Determine the scope from the user request: workflow run/job, PR, commit window, suite, or explicit local reproduction.
2. Read prior memory only when it is relevant to the named suite, signature, PR, or commit window; do not force daily automation history into a local run.
3. Pull the current failure log and identify the tested Rspack commit.
4. Compare against the user-provided baseline, previous green, prev release, or visible history when needed.
5. For every failing suite in scope, classify the conclusion as one of:
   - `reused`: same suite and same failure signature as prior memory.
   - `new-signature`: current signature differs from prior memory or baseline.
   - `new investigation`: no reliable prior conclusion exists for this scope.
   - `flaky/pre-existing`: the same signature predates the candidate PR or appears intermittently.
   - `inconclusive`: evidence is insufficient or conflicting.
6. Run Phase 1 for any suite the user asks to investigate or any `new-signature` / `new investigation` item.
7. Run Phase 2 only when Phase 1 finds a non-flaky candidate Rspack PR or version window with enough evidence. Do not run Phase 2 for known flaky, pre-existing, or downstream-only failures.
8. Use PR report comments only when `pr-report-comment.md` guardrails are satisfied and the user explicitly asks to comment. Otherwise report `no PR comment: <reason>`.

### Phase 1: PR Location

Goal: identify the actual source PR, date window, or downstream change that caused the suite to become red.

#### Fast-Exit Checks

Run these checks first before doing deep pivot analysis. If any check fires with high confidence, produce the Phase 1 output immediately and skip unnecessary steps.

1. **Same Rspack commit, different outcome**
   - If the exact same tested Rspack commit appears in both a green run and a red run of the same suite, the cause is **not** that Rspack commit.
   - Stop and attribute the failure to a downstream change, test expectation change, dependency update, or environment difference between the two runs.
   - Output example: `Actual source: downstream/test change (same Rspack SHA <sha> succeeded in run <green-id> and failed in run <red-id>)`.

2. **Known flaky or recurring failure signature**
   - Search prior run history, automation memory, and upstream commit/PR comments for the exact failure signature before blaming a new pivot.
   - If the same error appeared before the candidate PR was in the tested commit, classify it as `flaky` or `pre-existing recurring failure` unless new evidence proves the PR made it deterministic.
   - Example: `modernjs` `pure-esm-project` client navigation timeouts waiting for `#data` have appeared before; do not attribute that signature to a surface pivot just because the latest visible pivot looks plausibly related.
   - Output example: `Actual source: flaky/pre-existing (same selector timeout was reported before candidate PR #<n>)`.

3. **Surface PR diff is unrelated to the failure signature**
   - After fetching the surface PR, if its changed files and diff have no plausible connection to the observed error text, assertion, stack frame, or generated output, treat the surface PR as innocent.
   - Shift focus to downstream changes or an earlier Rspack commit that actually touched the failing path.
   - Output example: `Actual source: not surface PR #<n> (diff only touches <unrelated-paths>; failure signature is <signature>)`.

4. **Config-gated hypothesis not active**
   - If the attribution depends on a downstream option, plugin, loader, target, mode, or feature being enabled, verify that generated/runtime config actually enables it before using the PR as the actual source.
   - For Rsbuild-based suites, load [references/rsbuild-config-debug.md](references/rsbuild-config-debug.md) only when this check is needed.
   - Output example: `Actual source: not surface PR #<n> (generated config does not enable <required option>, but the hypothesis requires it)`.

5. **Failure signature directly maps to surface PR changed files and active config**
   - If the error text, failing command, or changed generated output directly involves files or APIs modified by the surface PR, the surface PR is likely the actual source.
   - For config-gated behavior, this only applies after confirming the required config is active.
   - Move to a lightweight Phase 2 to confirm the mechanism; do not spend time hunting alternative culprits.
   - Output example: `Actual source: surface PR #<n> (failure signature <signature> matches changed files <paths>)`.

Only continue with the full process below if none of the fast-exit checks gives a clear answer.

Use these evidence sources:

- Eco-ci status data, including current failed runs and previous green runs.
- GitHub Actions logs for current failure and candidate pivot failure.
- Rspack commit history, release tags, and canary versions.
- Downstream project history, dependency updates, snapshots, and test/config changes.
- Generated or runtime config only when the attribution hypothesis depends on config-gated behavior.
- `@rspack-canary/core` overrides in the downstream repo when the date or PR window is still too coarse.

Process:

1. Identify the run/job/PR/commit window in scope, the tested Rspack commit, and the nearest relevant comparison point when one is needed.
2. List failed suites in scope, failed count, run URL or run id, and the tested Rspack commit.
3. For each failed suite, find the green-to-red pivot in the visible history. If the same tested Rspack commit appears in both green and red runs, apply fast-exit check #1 and stop.
4. Pull logs for the current failure and the candidate pivot failure.
5. Check whether the same failure signature appeared in older runs, commit comments, PR comments, or automation memory before the candidate PR.
6. Compare failure signatures before attributing a root cause. After inspecting the surface PR diff, apply fast-exit checks #2 through #5 when the relationship between the diff and the signature is clear.
7. If the signature changed, search forward or binary-search red rows until the current signature appears.
8. Check whether the downstream project changed in the same window.
9. If the hypothesis depends on generated config, read the relevant config-debug reference and verify the active config before attributing the failure.
10. Reproduce enough combinations to decide whether the failure comes from Rspack, downstream, flaky infrastructure, or their interaction.
11. If release versions or eco-ci rows are too coarse, ask whether to run the canary date bisect tool.

Phase 1 output:

```text
Surface attribution: <PR shown by eco-ci pivot>
Actual source: <real PR, downstream PR, or version window>
Failure signature: <short signature>
Evidence: <run URLs, logs, canary results, or green/red pivots>
Confidence: high | medium | low
Notes: <why surface attribution is or is not responsible>
```

Only move to Phase 2 when there is a specific source PR or version window with enough evidence to inspect deeply.

### Phase 2: Deep Root Cause Debug

Goal: explain why the identified PR caused the observed behavior.

Use this phase after Phase 1 has identified a candidate source. Read [references/deep-pr-debug.md](references/deep-pr-debug.md) when the user asks for root cause, mechanism, or a fix direction.

Process:

1. Review the candidate PR metadata, commit, and diff.
2. Re-read the concrete failure log block and failing downstream assertion or stack.
3. Locate the downstream code path that produces the failure.
4. If the hypothesis depends on generated config, run the appropriate config-debug tool before claiming that the relevant option is active in the failing path.
5. Trace from downstream behavior into Rspack APIs, plugin hooks, loaders, generated output, source maps, runtime modules, or diagnostics.
6. Compare before/after behavior when needed, using canaries or local builds.
7. State the mechanical behavior change, not only the PR number.
8. Separate confirmed evidence from inference.

Phase 2 output:

```text
Candidate PR: <pr-number> <title>
Suite: <suite>
Verdict: caused | likely caused | not caused | inconclusive
Mechanism: <3-5 sentence explanation>
Evidence: <log URLs, code refs, reproduction results>
Confidence: high | medium | low
Next action: <fix in Rspack | fix downstream expectation | gather more evidence>
```

Use `gh` for specific job logs when available:

```bash
gh run view --job <job-id> --repo rstackjs/rstack-ecosystem-ci --log
```

For noisy logs, first isolate likely terminal failure blocks:

```bash
gh run view --job <job-id> --repo rstackjs/rstack-ecosystem-ci --log \
  | grep -E -i -C 3 'error|fail|panic|✖' \
  | head -200
```

Fall back to full logs when the filtered output misses the real failure.

## Reproduce Combination Relationships

Use combination testing in Phase 1 to separate Rspack changes from downstream changes.

Start with four conceptual combinations:

```text
old downstream + old Rspack
old downstream + new Rspack
new downstream + old Rspack
new downstream + new Rspack
```

Keep the downstream command fixed and use the narrowest failing command possible.

For finer Rspack windows, ask whether to use the canary date bisect tool, then follow [references/canary-date-bisect.md](references/canary-date-bisect.md).

### Downstream Interaction Check

If the downstream project changed during the same window, test these pairs when practical:

```text
old downstream + bad-window Rspack
old downstream + fixed Rspack
new downstream + bad-window Rspack
new downstream + fixed Rspack
```

This prevents wrongly attributing a downstream dependency/snapshot update to a later unrelated Rspack PR.

## Reporting Requirements

Keep reports compact and evidence-based:

- Name every currently failing suite.
- Include run URL or run id and tested Rspack commit when available.
- State whether each suite is newly investigated or reused from a matching known signature.
- Include the first visible start commit when there is a clear green-to-red pivot.
- Say when a failure predates the visible window.
- Include short log snippets only when they directly identify the failure.
- When surface attribution is misleading, explicitly say the surface PR is likely innocent and explain why.

For triage reports, use this structure for daily automation and local/manual runs:

```text
Scope: <latest completed status | workflow run/job | PR | commit window | suite>.
Current run/job: <run-url>, testing <sha> — "<commit msg>". Omit fields that do not apply.
Comparison baseline: <previous run | previous green | prev release | provided baseline | none>.
Failing suites in scope: <suite list>.
Delta: <new-suite/new-signature/recovered/unchanged/not applicable>.
PR comments: <posted links or "none: <reason>">.

### <suite> — <new investigation | reused | updated | flaky/pre-existing>

Attribution: <exact attribution line>

Root cause: <3-5 sentences. Separate confirmed facts from likely inference. If flaky or pre-existing, say so in the first sentence and do not force a PR root cause.>

Evidence: <log URLs actually read; include current and pivot/baseline URLs when used>
```

Use exactly one attribution line shape per suite:

```text
This failure started from <sha> — "<commit msg>". PR: <url>; author: <author>; date: <date>.
The prev release 66e23b5 was already failing with this same error.
This specific failure started from <sha> — "<commit msg>"; before that the suite was failing for a different reason already at prev release 66e23b5. PR: <url>; author: <author>; date: <date>.
This is a known flaky/pre-existing failure; the same signature appeared before <candidate-sha-or-pr>.
The current evidence is inconclusive; <candidate> is only a surface pivot because <missing-or-conflicting-evidence>.
```

Do not present a candidate PR as caused/likely-caused in the triage report unless the flaky-history check and any relevant config-gated check have both passed. Prefer `inconclusive` over a weak attribution.
