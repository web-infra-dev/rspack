---
name: rspack-perf-codspeed-goal
description: Use when optimizing a benchmarked code path through a GitHub PR where success depends on GitHub Actions checks, CodSpeed PR comments, requested percentage thresholds, flaky CI reruns, rebases, review comments, or repeated performance iterations. For Rspack performance work, combine this workflow with the rspack-perf skill and keep changes within the user-specified plugin, file, crate, feature, compilation stage, benchmark, or hot path unless expanding scope is necessary for correctness or the measured optimization.
---

# Rspack Perf CodSpeed Goal

Use this for performance work where success is judged by a GitHub PR, required correctness validation, review comments, and CodSpeed benchmark feedback.

The agent owns the loop. After the user gives the target benchmark, threshold, and scope, keep making the next best engineering decision without asking for routine confirmation. Continue until CodSpeed confirms the target and correctness gates are green, a real blocker prevents progress, or the configured round limit is reached.

## Invariants

- Treat explicit user targets as binding: benchmark name, required percentage, scope, PR, and maximum rounds.
- Default maximum is 10 optimization rounds unless the user specifies another limit.
- One round means one local optimization attempt plus local correctness validation, review validation, push, CI/CodSpeed validation, review-comment triage, and a continue-or-stop decision.
- When running under a persistent `/goal`, keep the original target intact across resumes. Do not redefine success around partial progress, a pending CI state, or a narrower benchmark.
- Do not end a non-terminal round without scheduling the required 10-minute CI/CodSpeed follow-up.
- Prefer the GitHub connector/plugin for GitHub operations; use `gh` only when the connector cannot perform the required action. Avoid the 1Password `gh` shell plugin unless explicitly requested.
- Preserve behavior unless the user explicitly accepts a semantic change.
- Keep changes scoped to the requested ownership boundary. Expand scope only when the measured bottleneck or correctness fix requires it, and explain the expansion.
- Use local verification only for correctness. Run all CodSpeed performance validation in CI and use the CI/CodSpeed result for the current PR head as the source of truth.
- Use append-only commits for normal optimization rounds. Do not routinely amend previous commits or force-push; use history rewriting only for an explicit user-requested rebase or another unavoidable repository operation.

## Initial Setup

1. Establish a clean base.
   - Check the working tree before changing code.
   - Stop if uncommitted user changes would be overwritten or mixed into the optimization.
   - Fetch latest `origin/main`.
   - If the user supplies an existing PR URL, branch, or head SHA, continue from that PR head unless they explicitly ask for a fresh branch or PR.
   - Otherwise use a clean branch or worktree based on fetched `origin/main`.
   - Confirm the optimization branch contains no unrelated commits or file changes.

2. Establish the target.
   - Record the benchmark name exactly as CodSpeed reports it.
   - Record the required threshold, for example `+5%` on a specific benchmark.
   - Record the requested optimization scope: plugin, file, crate, feature, compilation stage, benchmark, or hot path.
   - Record the PR URL, branch, base, head SHA, and current round number.
   - For Rspack work, use `$rspack-perf` to guide the optimization strategy before changing code.

3. Ensure PR-dependent actions wait for an open PR.
   - Reuse the supplied PR when one exists.
   - For a fresh branch, create the PR after the first scoped change is committed and pushed, before requesting review, creating the progress comment, or starting CI/CodSpeed tracking.

4. Create or identify the fixed progress comment after the PR exists.
   - Maintain one persistent PR comment that records every round and every pushed commit.
   - Reuse the existing progress comment if present; otherwise create one.
   - Update this comment after every pushed round and after every terminal CodSpeed decision.
   - Include only the round number, commit SHA, summary of code change, and CodSpeed result.

## Per-Round Checklist

Run these steps in order for every round.

1. Modify code locally.
   - Identify the next likely bottleneck from CodSpeed history, CI evidence, profiling artifacts, or code inspection.
   - Implement the smallest scoped change that plausibly improves the target benchmark.
   - Avoid unrelated cleanup, formatting churn, snapshot updates, or generated output unless required by the scoped change.
   - If a prior round failed due to correctness, review feedback, or CI, fix that before adding another performance idea.

2. Verify correctness locally.
   - Run targeted tests that cover the changed behavior.
   - Run the relevant lint/check command for the touched language, including `clippy` or Rust check coverage for Rust changes.
   - Run format verification, not broad formatting churn, unless formatting is already required.
   - For Rspack, follow repository guidance: JavaScript/TypeScript changes need the relevant JS build before JS tests; Rust changes need the relevant Rust binding build before Rust tests; mixed changes need the full dev build.
   - Do not run CodSpeed locally. All performance validation and comparisons must come from CI/CodSpeed for the current PR head.
   - Some local test cases, especially native watcher and swc-related cases, may be flaky in local runs. If such a known-flaky case fails and the failure is unrelated to the scoped change, record it as ignored local flakiness and continue; do not let it block the round.
   - If a local correctness command fails, fix it locally and repeat local verification before pushing.
   - Do not substitute CodSpeed performance data for correctness evidence.

3. Run a correctness-focused independent review.
   - Before every commit/push, review the current diff independently from the implementation pass.
   - Use a subagent only when it is available and explicitly allowed by the current environment and delegation rules; otherwise perform a local diff review.
   - Focus the review on correctness, behavior preservation, ordering, deduplication, hashing/equality, diagnostics, concurrency, cancellation, and cache lifetime where relevant.
   - If the review reports a real concern, fix it and repeat local verification plus independent review.
   - If the concern is not valid, record the technical reason in a review reply or final report.

4. Commit and push.
   - Review the diff for accidental unrelated edits.
   - Commit with a focused title, usually `perf: ...`.
   - Push the branch as a new commit on top of the prior round. Do not amend the previous optimization commit for normal iteration.
   - Use `--force-with-lease` only after an explicit rebase or another unavoidable history rewrite, not as the default round workflow.
   - If this is a fresh branch without a PR, create the PR now before any PR-dependent action.
   - Request or re-request GitHub Copilot code review for the latest head SHA.
   - Create or update the fixed progress comment with the new commit SHA and change summary.

5. Start CI/CodSpeed tracking.
   - Fetch workflow runs once after pushing to confirm CI started.
   - Create a 10-minute timer/follow-up automation to recheck GitHub CI status and the latest CodSpeed PR comment for the same PR head SHA.
   - Do not poll more frequently than the 10-minute cadence unless a result is already available and requires immediate action.
   - If automation tools are unavailable, report that blocker explicitly instead of relying on manual follow-up silently.

6. Classify CI status.
   - Track the full GitHub checks/task list for the current head SHA, not only CodSpeed.
   - Classify every CI task before judging success.
   - Treat CodSpeed checks as performance evidence and interpret them in the CodSpeed step.
   - Record `Binary Size Limit` separately; do not block correctness triage on it unless the user or repository policy makes it required.
   - All test-related CI actions must pass before CodSpeed feedback is trusted for success or next-round performance decisions.
   - All other required correctness CI tasks must pass before success can be reported.
   - For any non-CodSpeed, non-Binary-Size failure, fetch job details and logs, analyze cause, and decide independently:
     - If caused by the optimization or fixable in scope, fix locally and start a new round.
     - If clearly unrelated and known flaky, rerun only the failed job or jobs once, then recheck the same head SHA.
     - If caused by stale generated output or version mismatch, update the relevant source/generated files locally and start a new round.
   - For any failed test-related action, fetch the failure logs, identify the failing test or build step, fix the cause when it is related or plausibly related to the optimization, and start a new round. Do not treat CI as trustworthy while any test-related action is failed, cancelled, missing, or still pending.
   - If CI is still pending, create another 10-minute follow-up before yielding.

7. Handle code review comments before performance evaluation completes.
   - Inspect unresolved code review comments and review threads for the latest PR head SHA before declaring the performance evaluation complete.
   - Include Copilot and human reviewer comments.
   - Inspect outdated threads against the current code and resolve them only after confirming the concern was addressed or no longer applies.
   - If a live comment is reasonable, implement the fix, reply with what changed, resolve the thread, and start a new round with local verification and independent review.
   - If a live comment is not reasonable, reply with the technical reason and supporting evidence, resolve the thread, and continue the same evaluation if no code changed.
   - Do not declare success while any non-outdated review comment or requested-change thread remains unresolved.

8. Read and interpret CodSpeed feedback.
   - Use the latest CodSpeed PR comment for the current head SHA.
   - Before using CodSpeed to judge success or choose the next performance direction, confirm all test-related CI actions passed for the same head SHA.
   - If any test-related CI action has not passed, record CodSpeed as provisional only, fix or wait on CI first, and do not declare the target reached.
   - Compare the requested benchmark against the PR base used by CodSpeed.
   - Record exact before/after values, percentage delta, environment warnings, and whether the requested threshold is met.
   - Compare the current commit's performance result with the most recent retained snapshot's result when both are available.
   - Check for meaningful regressions in other benchmarks before declaring success.
   - Treat environment warnings as reportable context. Discard a result only when the comparison is unreliable.
   - Update the fixed progress comment with the CodSpeed result.

9. Decide the next action independently.
   - If local correctness, required CI, review comments, and CodSpeed target all pass, report success with exact commit SHA, benchmark values, percentage delta, and PR status.
   - Compare the first candidate with the CodSpeed PR base. Compare every later candidate with the most recent retained snapshot, including a retained correctness-only fix, because that snapshot is the candidate's actual code parent.
   - If a performance-focused round does not improve performance versus that comparison point, revert that commit's code changes with a new follow-up commit and update the progress comment with the reverted state. If the round limit has been reached, stop after the revert; otherwise start the next round from the reverted code state.
   - If a round only fixes correctness, CI, or review feedback, keep the fix when validation improves even if the CodSpeed result is neutral or slower. Record its trusted CodSpeed result as the comparison point for the next candidate.
   - If CodSpeed improves versus that comparison point but remains below threshold and the round limit is not reached, immediately choose the next scoped optimization direction and start the next round.
   - If review or CI requires code changes, start the next round focused on that fix before additional performance work.
   - If the round limit is reached, stop and report the best observed result, all attempted directions, current CI/review state, and the reason for stopping.
   - If resumed from a timer, goal continuation, or thread wakeup, inspect the current PR head SHA, fixed progress comment, CI state, review threads, and latest CodSpeed comment before deciding. Continue the same round when the head SHA is unchanged; start a new round only when code changes are required.
   - Ask the user only for a true product or semantic decision, missing credentials/access, or a blocker that cannot be resolved through code, CI, GitHub, or review analysis.

10. Perform terminal CodSpeed explain request.

    - Run this only after the workflow is terminal: either the target is reached or the round limit has been reached.
    - If the final trusted CodSpeed result shows a net performance improvement over the CodSpeed PR base, add a PR comment exactly: `@codspeedbot explain why this PR is faster`.
    - Add this comment even when the target threshold was not reached, as long as the final trusted CodSpeed result is faster than the PR base.
    - Do not add the comment when the final CodSpeed result is provisional, unreliable, neutral, or slower than the PR base.
    - Record whether this comment was added in the fixed progress comment and final report.

## Progress Comment Format

Keep one fixed PR comment in this shape and refresh it throughout the work:

```markdown
## CodSpeed Optimization Log

Target: `<benchmark>` >= `<threshold>`
Scope: `<requested scope>`
Current status: `<running|passed|failed|blocked>`
Latest head: `<sha>`

| Round | Commit  | Change      | CodSpeed             |
| ----- | ------- | ----------- | -------------------- |
| 1     | `<sha>` | `<summary>` | `<delta or pending>` |
```

Update, do not duplicate, this comment. Keep entries concise but specific enough that the commit history and performance result history are visible.

## Rspack Scope

When the repository or task is Rspack-specific:

- Use `$rspack-perf` before implementation to guide the optimization strategy.
- Treat the user's requested plugin, file, crate, feature, compilation stage, or benchmark as the ownership boundary.
- Optimize from Rspack's data cardinality model: dependencies and export infos are usually higher risk than modules, chunks, chunk groups, entries, or runtimes.
- Prefer focused CPU and allocation reductions in the requested area over broad cleanup or unrelated refactors.
- Do not touch unrelated plugins, compilation stages, tests, snapshots, or formatting outside the changed files unless required by the scoped optimization or CI result.
- If the measured bottleneck crosses the requested boundary, explain the reason before expanding scope and keep the expansion minimal.

## Correctness Guardrails

- When caching derived state, verify that the state is stable for the whole pass where it is reused.
- When adding a fast path, compare it against the original slow path for all relevant modes, not only the common case.
- When changing data structures to reduce allocation, check ordering, deduplication, hashing, equality, and user-visible diagnostics.
- When changing concurrency or scheduling, check determinism, cancellation, shared-state ownership, and cleanup.
- When changing build artifacts or generated files, verify source and generated output stay consistent.
- If a subagent, reviewer, CI job, or local command finds a correctness concern, either fix it or document why code/tests prove it harmless.

## Reporting

Keep updates short and decision-oriented:

- State the PR URL, branch, head SHA, round number, and target.
- State what changed in the current round and why it should affect the benchmark.
- List local correctness validation: tests, clippy/check/lint, format verification, ignored local flaky cases, and independent review result.
- List required CI checks that passed, failed, or are pending.
- State Copilot/human review-comment status and how comments were handled.
- Summarize CodSpeed outcome with exact benchmark deltas and target pass/fail.
- State whether the fixed progress comment was updated.
- State whether `@codspeedbot explain why this PR is faster` was posted when the terminal CodSpeed result had an improvement.
- State whether the next 10-minute CI/CodSpeed timer was created, unless the workflow is complete or immediately continuing into another local round.
- If more work is needed, name the next optimization or correctness direction.
