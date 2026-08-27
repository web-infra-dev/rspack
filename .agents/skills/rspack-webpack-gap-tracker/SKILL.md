---
name: rspack-webpack-gap-tracker
description: Use when tracking, auditing, or updating Rspack compatibility gaps against webpack, including missing webpack features, built-in and ecosystem plugin compatibility gaps, completed compatibility features, and missing webpack test coverage.
---

# Rspack Webpack Gap Tracker

## Tracking Targets

- Main tracking issue: https://github.com/web-infra-dev/rspack/issues/14556
- Completed feature comment ID: `4785525873`
- Missing test comment ID: `4785526309`
- Native CSS tracking issue: https://github.com/web-infra-dev/rspack/issues/14002

## Required Output Shape

Keep the issue and comments as lists, not prose-only summaries.

- Main issue body: missing Rspack features compared with webpack, grouped by implementation recommendation.
- Completed feature comment: webpack compatibility features that Rspack has completed.
- Missing test comment: webpack tests missing from Rspack that are not already covered by a missing feature or PR-specific feature gap.

Keep workflow and update rules in this skill only. Do not add an `Update Rules` section or other agent instructions to the GitHub issue body or placeholder comments.

Every update to the issue body or either placeholder comment must include a visible update time, using the local timezone when possible:

```text
Last updated: YYYY-MM-DD HH:mm:ss Z
```

## Preview and Approval Gate

Before every GitHub update, generate a local Markdown preview containing the exact replacement body for the tracking issue and every placeholder comment that would be changed. Clearly mark the file as a preview, include its local path in the response, and do not call any GitHub write operation yet.

Wait for the user to explicitly approve that preview before updating GitHub. After approval, re-read the current issue and comments and compare them with the state used to generate the preview. If GitHub changed in the meantime, do not apply the stale preview; generate a refreshed local preview and request approval again.

Apply only the approved preview content. Do not add, remove, reclassify, or reword entries during the write step. A request that both approves the current preview and asks to update the issue satisfies this gate for that preview.

## Audit Workflow

1. Read the current tracking issue and its comments before adding anything.
2. Preserve the existing issue and comment format when updating; only change the relevant timestamp and list entries.
3. Check associated, linked, or subtask issues for each candidate gap. If a related issue or subtask already records the same content, do not duplicate it in the main tracking issue.
4. Compare new findings with both missing and completed lists to avoid duplicates.
5. Search for missing functionality from these directions:
   - New webpack release features since the last recorded update.
   - Differences between webpack's config object and Rspack's config object.
   - Differences between Rspack built-in plugins and webpack built-in plugins, plus commonly-used webpack ecosystem plugins that Rspack provides built-in alternatives for.
   - Rspack PRs merged since the last recorded update, especially large changes and PRs labeled or titled as features.
   - Existing issue entries and completed entries, to avoid re-recording already tracked work.
6. Put every missing feature in **Pending evaluation** by default. Do not infer a recommendation from implementation details, performance considerations, existing Rspack APIs, user-demand evidence, or whether the feature could live in an external package.
7. Put or move any gap into **Recommended to implement in Rspack** only when the tracking issue already contains an explicit manual annotation marking that gap as recommended. Preserve that manual classification on later audits. The audit's own proposed wording or reasoning does not count as a manual recommendation; when the provenance is unclear, leave the gap in **Pending evaluation**. Do not add new gaps to **Not recommended to implement in Rspack** automatically; preserve existing manual classifications.
8. Inspect the corresponding webpack feature or plugin implementation for accurate compatibility notes. Check whether Rsbuild already provides a solution, and whether https://github.com/rstackjs already has a compatible package or plugin, but keep gaps in **Pending evaluation** unless the issue has the manual recommendation annotation described above.
9. If the gap is native CSS-related, record it in #14002 instead of #14556.
10. For missing tests:
   - If a missing webpack test maps clearly to a tracked feature gap or a specific PR gap, mention it under that feature instead of adding it to the standalone missing-test comment.
   - Otherwise, list missing tests by webpack test directory.
   - Include older missing tests directly; do not hide them just because they predate the last update.

## Suggested Investigation Sources

- Webpack releases and changelog for new feature candidates.
- Webpack config schema/types and Rspack config schema/types for option-level gaps.
- Webpack built-in plugin docs/source, Rspack built-in plugin exports/source, and webpack ecosystem plugin docs/source for plugin-compatibility gaps.
- Webpack feature/plugin implementation details for accurate compatibility notes; this investigation does not change the default **Pending evaluation** classification.
- GitHub merged PRs in `web-infra-dev/rspack` since the last update.
- Existing Rspack issues, linked issues, subtask issues, and the completed-feature comment for deduplication.
- Rsbuild docs/source and https://github.com/rstackjs packages for existing compatibility solutions.
- `tests/rspack-test/` and the local webpack checkout, when available, for test coverage comparison.

## Compare Test Directories

Run the bundled zx script before manually auditing missing webpack tests. It compares canonical test case directories and maps webpack's `test/cases` to Rspack's `tests/rspack-test/normalCases` and webpack's `test/statsCases` to Rspack's `tests/rspack-test/statsOutputCases`.

Run it through the skill workspace package:

```bash
pnpm --filter @rspack/skill-webpack-gap-tracker diff-tests -- --webpack /path/to/webpack
```

Pass positional filters like Rstest file filters. Treat each filter as a case-insensitive regular expression against a webpack-style test path; combine multiple filters with OR:

```bash
pnpm --filter @rspack/skill-webpack-gap-tracker diff-tests -- --webpack /path/to/webpack configCases/asset
pnpm --filter @rspack/skill-webpack-gap-tracker diff-tests -- --webpack /path/to/webpack '^(config|hot)Cases/css'
```

Add `--content` to emit unified file-content patches for test cases present in both repositories. Always pass a positional filter with content diff unless a full-repository comparison is intentional:

```bash
pnpm --filter @rspack/skill-webpack-gap-tracker diff-tests -- --webpack /path/to/webpack --content '^configCases/asset-modules/url-relative$'
```

Treat `webpack.config.{js,mjs,cjs,ts,mts,cts}` and `rspack.config.{js,mjs,cjs,ts,mts,cts}` as the same logical file when comparing content, even when their extensions differ. Keep their real names in diff headers. Use `--context <lines>` to control unified diff context.

Normalize supported text formats with one deterministic Prettier configuration before comparing them so quote, indentation, trailing-comma, line-ending, and trailing-whitespace differences do not appear in patches. If parsing fails, fall back to line-ending and trailing-whitespace normalization. Use `--no-format` only when the exact raw text difference is required.

Set the webpack checkout with `--webpack` or `WEBPACK_ROOT`. Set a different Rspack checkout with `--rspack`. Pass either a repository root or its test directory.

Use `--direction webpack` to show tests missing from Rspack, `--direction rspack` to show Rspack-only tests, or the default `--direction both` to show the symmetric directory diff. Read `scripts/diff-tests.mjs` only when changing its comparison behavior.

## Entry Format

Use concise checklist entries. Prefer one feature or test group per item.

Missing feature entry:

```markdown
- [ ] `<feature or option>` - Short compatibility note. Recommendation: pending by default; use recommended only when preserving an explicit manual annotation already present in the tracking issue. Source: webpack release/config/PR link.
```

Completed feature entry:

```markdown
- [x] `<feature or option>` - Completed in Rspack via PR/commit/issue link.
```

Missing test entry:

```markdown
- [ ] `<webpack test directory>` - Missing notable cases: `case-a`, `case-b`. Related feature: none.
```

If a source is uncertain, mark it as needing verification instead of presenting it as confirmed.
