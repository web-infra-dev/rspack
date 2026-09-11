# Automation Daily Triage

Use this reference for recurring Rstack ecosystem CI automation runs and latest-status monitoring tasks.

## Trigger

Read this file before inspecting runs when the request:

- Mentions daily triage, automation, latest/today's Rstack ecosystem CI status, or `/goal`.
- Provides an automation id, automation memory path, or last-run timestamp.
- Requires sending or delivering the report to a user/chat.
- Asks for all currently failing suites in one or more supported ecosystems.

For a single job, PR, suite, or local reproduction request, use the local quick path in `SKILL.md` instead.

## Scope

Resolve the ecosystem set explicitly. Supported values are `rspack`, `rsbuild`, `rslib`, `rspress`, `rstest`, and `rsdoctor`.

- If the automation names ecosystems, inspect exactly that set.
- If it asks for all Rstack ecosystem CI, inspect all six status sources. Do not stop after `rspack.json`.
- For all-ecosystem triage, inspect, live-verify, and report `rspack` first because it is the primary and highest-volume ecosystem; then cover the remaining selected ecosystems.
- Keep results grouped by selected ecosystem. The upstream commit comes from that ecosystem's status row; the failing suite is a downstream target and must not replace the upstream identity.

## Process

1. Read the automation memory first. Use it to distinguish reused signatures from new suites or changed signatures, keyed by `<ecosystem>/<suite>` rather than suite name alone.
2. Refresh `origin/data` when a local `rstack-ecosystem-ci` checkout is available.
3. For every selected ecosystem, run `scripts/ecosystem-status.sh --ecosystem <ecosystem> --repo <ecosystem-ci-path>` or pass a matching local JSON snapshot. This selects the latest and previous rows from `origin/data:<ecosystem>.json`; do not use an in-progress workflow as completed status.
4. Cross-check each selected latest row against its live workflow run and concrete suite jobs. A status row's `commitSha` is the tested upstream commit; an ecosystem-ci workflow `headSha` is not.
5. List every currently failing `<ecosystem>/<suite>` pair, job URL, run URL, and tested upstream commit.
6. Compare latest versus previous completed row per ecosystem:
   - `reused`: same ecosystem, suite, and failure signature as memory/current sampled log.
   - `new-suite`: suite is failing now but was not failing in that ecosystem's previous completed row.
   - `new-signature`: suite was already failing, but the current failure signature changed.
   - `recovered`: suite was failing previously but is no longer current; mention only in the summary.
7. For reused failures, sample the current log enough to confirm the signature still matches memory before reusing the conclusion.
8. For `new-suite` or `new-signature`, run Phase 1. Run Phase 2 only when Phase 1 finds a non-flaky candidate PR or version window in the selected upstream with enough evidence.
9. If the automation asks to comment on PRs, read `pr-report-comment.md` and comment only when its guardrails are satisfied. Otherwise include `no PR comment: <reason>` in the report.
10. Perform a final freshness check for every selected ecosystem before delivery. A newer completed row supersedes the earlier target for that ecosystem; an in-progress row does not.
11. If the automation asks to deliver the report, send it only after the report content is final.
12. Update automation memory with:
    - current run time and selected ecosystem set,
    - latest and previous run ids/URLs per ecosystem,
    - current failing `<ecosystem>/<suite>` set,
    - per-pair signature and attribution,
    - delivery message id, if any,
    - PR comment links, if any.

## Report Notes

Start with a compact matrix summary. Every selected ecosystem must appear, including green ecosystems:

```text
Ecosystem | Latest run | Tested commit | Result | Failing suites | Delta
rspack    | <url>      | <sha>         | green  | none           | unchanged
rsbuild   | <url>      | <sha>         | green  | none           | unchanged
rslib     | <url>      | <sha>         | red    | rspress        | unchanged
```

Then use the triage report contract from `SKILL.md` for every current failure:

```text
Scope: <ecosystem>; latest completed status.
Current run/job: <latest completed run-url>, testing <upstream sha> — "<commit msg>".
Comparison baseline: previous completed run <run-url>, testing <upstream sha>.
Failing suites in scope: <suite list>.
Delta: <new-suite/new-signature/recovered/unchanged summary>.
PR comments: <posted links or "none: <reason>">.
```

Every currently failing `<ecosystem>/<suite>` pair gets its own short section. Do not include recovered suites as failing-suite sections.
