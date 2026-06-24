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

Every update to the issue body or either placeholder comment must include a visible update time, using the local timezone when possible:

```text
Last updated: YYYY-MM-DD HH:mm:ss Z
```

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
6. Classify each missing feature into one of these recommendation buckets:
   - Recommended to implement in Rspack: gaps in config or plugin functionality that Rspack already exposes, or features that need native Rust/compiler integration for correctness or performance.
   - Pending evaluation: gaps where the right home is unclear, or where webpack support may be experimental, low priority, or partially covered by existing Rspack/Rsbuild behavior.
   - Not recommended to implement in Rspack: plugin or integration features that can be implemented outside Rspack without native Rust work or performance impact; prefer a third-party package or an independent plugin under https://github.com/rstackjs when the compatibility layer should be maintained by the Rstack ecosystem.
7. Before marking a feature as not recommended or pending, inspect the corresponding webpack feature or plugin implementation. Check whether Rsbuild already provides a solution, and whether https://github.com/rstackjs already has a compatible package or plugin.
8. If the gap is native CSS-related, record it in #14002 instead of #14556.
9. For missing tests:
   - If a missing webpack test maps clearly to a tracked feature gap or a specific PR gap, mention it under that feature instead of adding it to the standalone missing-test comment.
   - Otherwise, list missing tests by webpack test directory.
   - Include older missing tests directly; do not hide them just because they predate the last update.

## Suggested Investigation Sources

- Webpack releases and changelog for new feature candidates.
- Webpack config schema/types and Rspack config schema/types for option-level gaps.
- Webpack built-in plugin docs/source, Rspack built-in plugin exports/source, and webpack ecosystem plugin docs/source for plugin-compatibility gaps.
- Webpack feature/plugin implementation details when deciding whether the gap belongs in Rspack, an external package, or an Rstack ecosystem plugin.
- GitHub merged PRs in `web-infra-dev/rspack` since the last update.
- Existing Rspack issues, linked issues, subtask issues, and the completed-feature comment for deduplication.
- Rsbuild docs/source and https://github.com/rstackjs packages for existing compatibility solutions.
- `tests/rspack-test/` and the local webpack checkout, when available, for test coverage comparison.

## Entry Format

Use concise checklist entries. Prefer one feature or test group per item.

Missing feature entry:

```markdown
- [ ] `<feature or option>` - Short compatibility note. Recommendation: recommended/pending/not recommended, with a short reason. Source: webpack release/config/PR link.
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
