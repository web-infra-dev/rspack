---
name: rspack-pr-review
description: Review a pull request in web-infra-dev/rspack from a PR number or GitHub PR URL, classify the change, inspect its design, implementation, tests, compatibility, correctness and performance risks, compare relevant behavior with webpack or other bundlers, and publish or update one concise bilingual English/Chinese review report as a PR comment. Use for Rspack PR design reviews, implementation reviews, risk assessments, and requests to produce or publish a structured PR review report.
---

# Review an Rspack PR

## Input and scope

- Require either a numeric PR ID or a GitHub PR URL.
- Interpret a numeric ID as `web-infra-dev/rspack#<id>`.
- Reject URLs for another repository unless the user explicitly asks to review that repository with this workflow.
- Treat the user's request to use this skill as authorization to create or update the final report as one ordinary PR comment. Do not approve, request changes, merge, close, label, or otherwise mutate the PR.
- If the PR cannot be read or commented on, provide the complete comment body to the user and state the exact blocker.

## Gather evidence

Use the connected GitHub tools when available; otherwise use authenticated `gh` and local `git`. Inspect evidence rather than relying only on the PR description.

1. Resolve the PR number, repository, base SHA, head SHA, author, title, body, labels, commits, changed files, checks, review threads, issue comments, and review comments.
2. Read the complete diff. For large PRs, inspect every changed file at least at the structural level, then deeply inspect behavior-bearing code, public types, tests, documentation, benchmarks, fixtures, snapshots, and generated changes.
3. Inspect the relevant base-branch implementation and history to explain the previous design and why the change is necessary. Follow repository `AGENTS.md` instructions.
4. Trace important flows across Rust and JavaScript/TypeScript boundaries where applicable. Check error paths, caching, invalidation, ordering, concurrency, determinism, platform differences, and lifecycle behavior.
5. Inspect whether existing and changed tests cover the trigger or new behavior, meaningful edge cases, regressions, and API compatibility. Use test design and verified remote CI results as evidence, but do not run local builds, tests, benchmarks, linters, formatters, or static checks. Assess correctness from the code paths, state transitions, invariants, and failure modes themselves.
6. For performance PRs, read PR comments and check results for `ecosystem-benchmark` and CodSpeed evidence. Distinguish measured results from author claims and missing data.
7. For feature work, search webpack's current source, tests, and documentation for analogous behavior. If none exists, inspect another relevant bundler such as Rollup, esbuild, Parcel, or Vite. Cite stable source links in the report when useful. Do not infer parity from naming alone.

Fetch remote information when it may have changed. Prefer primary evidence: repository code, tests, official documentation, benchmark output, and CI results.

## Classify the PR

Choose one primary type based on the dominant intent and implementation:

- **Feature development**: adds or materially changes user-visible behavior, configuration, hooks, APIs, or capabilities.
- **Bug fix**: corrects behavior that violates the intended contract in a reproducible scenario.
- **Performance optimization**: primarily reduces CPU, memory, allocation, I/O, build time, or rebuild time while intending to preserve behavior.
- **Other**: refactoring, project governance, dependency/build/CI work, documentation, tests, maintenance, or mixed work without a dominant category above.

Mention important secondary aspects in the overview. Do not force a governance or refactor PR into a feature category merely because code changes.

## Evaluate findings

- Separate verified defects from plausible risks, questions, and missing evidence.
- For every material concern, identify the affected code path and a concrete trigger. Explain impact and why existing tests do or do not cover it.
- Judge correctness primarily by tracing the implementation logic through concrete inputs and lifecycle transitions. Treat tests as coverage evidence, not as a substitute for understanding why the code is correct or incorrect.
- Avoid speculative filler. State `No material risk identified from the inspected changes` when appropriate, while noting validation limits.
- Treat API changes broadly: JavaScript/TypeScript types, Rust-facing contracts, configuration schema, hooks, defaults, diagnostics, output shape, ordering, serialization, and documented behavior.
- Call a change breaking only when existing valid usage or observable behavior can fail or change incompatibly. Explain the compatibility boundary.
- Evaluate performance directionally even without benchmarks: hot-path work, algorithmic complexity, allocations, cloning, hashing, locking, I/O, cache hit rate, and parallelism.
- Keep the report focused on design and risk. Do not turn it into a file-by-file changelog.

## Write the report

Produce a visible English report followed by a semantically equivalent Chinese translation contained entirely inside a collapsed block. Keep each section short but explicit, normally one to three compact paragraphs or a few bullets. Use `Not applicable`, `Not found`, or `Not verified` instead of omitting a required topic.

Do not place any Chinese text before the opening `<details>` tag. The reviewed-commit line, visible disclaimer, headings, prose, and collapsed-block summary must all be English; Chinese may appear only inside the collapsed block's body. Do not add language labels such as `English`, `Chinese`, `英文`, or `中文` anywhere in the comment.

Start the comment with this preamble, replacing `<full head SHA>` with the complete PR head commit SHA that was actually reviewed:

```markdown
**Reviewed commit:** `<full head SHA>`
<!-- rspack-pr-review -->

**This comment is only intended to assist code review and does not constitute a direct code review recommendation.**
```

The reviewed-commit line must be the first visible line, and the hidden marker must remain unchanged so later runs can find and update the comment. Then write the visible report directly, without a language-version heading. Select only the outline matching the primary type.

### Feature development outline

1. `### PR Overview`
2. `### Previous Design`
3. `### New or Changed Design`
4. `### webpack and Other Bundlers`
   - State whether webpack has an analogous implementation.
   - If yes, explain concrete differences.
   - If no, assess a relevant alternative bundler.
5. `### API and Documentation`
   - Identify API changes, breaking changes, and documentation coverage.
6. `### Correctness Risks`
7. `### Performance Regression Risks`

### Bug-fix outline

1. `### PR Overview`
2. `### Problem and Trigger`
3. `### Previous Design and Root Cause`
4. `### Fix Design`
5. `### Correctness Risks`
6. `### Performance Regression Risks`

Explain why the fix addresses the root cause, not merely what lines changed.

### Performance-optimization outline

1. `### PR Overview`
2. `### Optimized Phase`
3. `### Previous Design and Bottleneck`
4. `### New Design and Why It Is Faster`
5. `### Benchmark Evidence`
   - Report ecosystem-benchmark and CodSpeed evidence separately, including absence or inconclusive results.
6. `### Correctness Risks`

### Other outline

1. `### PR Overview`
2. `### Previous Design` — include only when existing behavior or architecture is changed.
3. `### Current Design`
4. `### Correctness Risks`
5. `### Performance Regression Risks`

After the visible report, put the entire translation in a collapsed block without a language-version heading:

```text
<details>
<summary>Translation</summary>

**本评论仅用于辅助代码审查，不构成直接的代码审查建议。**

...translated report with the same type-specific structure...

</details>
```

Use natural translated headings corresponding to the visible headings, but do not add a heading that names either language. Preserve technical identifiers in their original form. Ensure the two versions make the same claims, risk assessments, and confidence qualifications. Keep every Chinese character, including the translated disclaimer, after `<details>` and before `</details>`.

## Post and verify

1. Draft the complete comment in a temporary file to avoid shell-quoting corruption.
2. Immediately before publication, resolve the current PR head SHA again. If it differs from the head SHA used for the review, inspect the new changes and regenerate the report; never attach a stale report to the new SHA.
3. Put the complete reviewed head SHA in the first visible line. Recheck every factual claim against collected evidence, confirm both language versions match, verify that the hidden marker is present, verify that no Chinese character appears before `<details>`, and ensure the comment contains no language-version heading or label.
4. List the pull request's existing issue comments. Match a workflow comment by the exact `<!-- rspack-pr-review -->` marker; for comments created before this marker existed, also match a comment whose first line is the exact disclaimer. If multiple comments match, select the most recently updated one.
5. If a workflow comment exists, verify that the authenticated account can edit it and update that comment in place through its issue-comment endpoint. If it cannot be edited, report the blocker and do not create a duplicate. Create exactly one ordinary issue comment only when no workflow comment exists.
6. Read back the created or updated comment to verify its body, reviewed head SHA, formatting, URL, and successful publication.
7. Return the PR link, comment link, whether the comment was created or updated, primary classification, remote checks inspected, and any evidence limitations to the user. Do not report local validation because this workflow must not run it.

Do not post partial drafts or multiple correction comments. If a create or update operation has an uncertain result, fetch the matching comment and compare its body before retrying.
