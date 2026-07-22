---
name: rspack-perf-valgrind-goal
description: Use when optimizing a benchmarked Rspack code path against a requested percentage target with reproducible local Docker and Valgrind measurements instead of GitHub Actions or CodSpeed cloud results. Combine this workflow with the rspack-perf skill for repeated performance rounds, local correctness validation, review-comment handling, retained-versus-reverted decisions, and PR progress reporting while keeping changes within the user-specified plugin, file, crate, feature, compilation stage, benchmark, or hot path.
---

# Rspack Perf Valgrind Goal

Use this for Rspack performance work where success is judged by the Callgrind instruction count of an exact benchmark stage measured locally in a pinned native-architecture Docker environment. Build the existing Rust benchmark target with ordinary Cargo, execute the selected binary directly under Valgrind, and decide only from local Callgrind profiles. Do not run a CodSpeed CLI, use a token, upload results, read CodSpeed PR comments, or wait for GitHub Actions.

The agent owns the loop. After the user gives the target benchmark stage, threshold, and scope, continue until local Valgrind confirms the target and correctness validation passes, a real blocker prevents progress, or the configured round limit is reached.

## Invariants

- Treat explicit user targets as binding: exact measured stage, required percentage, scope, PR, and maximum rounds.
- Default to at most 10 optimization rounds unless the user specifies another limit.
- One round means one optimization attempt, local correctness validation, correctness-focused review, a committed snapshot, local Docker/Valgrind measurement, a retain-or-revert decision, and progress reporting.
- For Rspack work, use `$rspack-perf` to guide optimization strategy before changing code.
- Preserve behavior unless the user explicitly accepts a semantic change.
- Keep changes within the requested ownership boundary. Expand scope only when the measured bottleneck or a correctness fix requires it, and explain the expansion.
- Use only local measurements from the same native Docker platform, image ID, fixture directory, benchmark target, benchmark filter, environment, and measurement mode for comparisons.
- A usable run must execute only the intended benchmark filter and produce a Callgrind profile with a nonzero `instruction_count`. Benchmark harness labels, cloud percentages, and wall time are not performance evidence for this workflow.
- Do not use GitHub Actions status or CodSpeed PR comments to judge performance or correctness. CI may run after a push, but this workflow neither waits for nor monitors it.
- Use append-only commits for normal rounds. Do not routinely amend or force-push; rewrite history only for an explicit rebase or another unavoidable repository operation.
- When running under a persistent `/goal`, retain the original target across resumes. Do not redefine success around partial progress.

## Local Measurement Contract

Use `scripts/run_local_valgrind.sh` from this skill. It builds a reusable native-architecture image containing the repository's pinned Rust toolchain and Debian's standard Valgrind package. The helper mounts source and canonical fixtures read-only, stores build caches in Docker volumes keyed by checkout path, image ID, and native platform, builds with ordinary Cargo, executes only the selected benchmark binary under Callgrind, captures raw logs and profiles, and records an environment manifest.

For persistent-cache benchmark filters, the helper copies the read-only fixtures into ephemeral writable container storage before execution because those benchmarks create temporary workspaces beneath `RSPACK_BENCHCASES_DIR`. The copy preserves fixture symlinks and is discarded with the container; never make the canonical host fixture directory writable.

The helper passes the repository's existing `--cfg codspeed` compile-time switch to ordinary `cargo build` so the benchmark adapter exposes its Valgrind client-request boundaries. This is only a source configuration name: no CodSpeed executable, service, token, API, upload, PR comment, or benchmark result participates in the workflow. Valgrind starts instrumentation at the selected benchmark boundary, and `run-*.instructions` plus the local Callgrind profiles are the only performance source of truth.

## Docker Prerequisite and Installation

Verify Docker before preparing fixtures or building the measurement image:

```bash
command -v docker
docker version
docker info
docker compose version
docker run --rm hello-world
```

If Docker is missing or the engine is unavailable, install and validate it before continuing. When the local task history is available, read task `019f7e27-50ba-7ac1-8ab4-3cdc868818fe` (`安装 Docker 并运行 Valgrind`) and follow its tested installation flow. Do not replace the Docker measurement environment with host-side Valgrind.

Apply these installation requirements even when that task cannot be read:

- Detect the host OS and native architecture before downloading anything. On Apple Silicon macOS, install the official arm64 Docker Desktop application, start Docker Desktop, and wait for the Linux engine to become ready.
- Treat license acceptance, administrator privileges, and first-launch permission dialogs as user-owned gates. Ask the user to complete them when the operating system requires interaction; do not bypass them.
- Ensure the Docker CLI, Compose plugin, and credential helper are reachable from the current shell. If Docker Desktop was installed by copying the application and did not update `PATH`, add its supported CLI directory to the shell configuration, start a new shell, and re-run the verification commands above.
- Use `docker run --rm hello-world` to verify image pulling, credentials, networking, and container execution rather than accepting an installed application as sufficient evidence.
- On macOS, allocate about 16 GiB to Docker Desktop when the host has enough memory. The Rspack optimized benchmark build has been observed to receive `SIGKILL` with the default 8 GiB allocation. If that happens, increase Docker memory, restart the engine, preserve the Docker volumes, and resume the cached build.
- Confirm the organization permits Docker Desktop use under its applicable commercial license. On Linux, install the official Docker Engine packages appropriate for the distribution and apply the same engine and `hello-world` verification.

From the optimization checkout, initialize the environment once:

```bash
SKILL_DIR="<absolute path to .agents/skills/rspack-perf-valgrind-goal>"
"$SKILL_DIR/scripts/run_local_valgrind.sh" build-image
"$SKILL_DIR/scripts/run_local_valgrind.sh" prepare --repo "<optimization checkout>"
```

Prepare fixtures only once for a goal. Pass that exact fixture directory to both base and candidate measurements:

```bash
"$SKILL_DIR/scripts/run_local_valgrind.sh" measure \
  --repo "<checkout to measure>" \
  --fixtures "<optimization checkout>/.bench/rspack-benchcases" \
  --bench "<benches or rspack_sources>" \
  --filter "<exact benchmark filter>" \
  --output "<absolute result directory>"
```

The helper defaults to two repetitions. Read each exact instruction count from `run-*.instructions`. If the stage differs by more than 0.5% between repetitions, run a third repetition with `--repeat 3` and use the median. Keep raw logs and Callgrind profiles; do not copy a number manually without retaining its source artifact.

## Initial Setup

1. Establish a clean optimization checkout.
   - Check the working tree before changing code.
   - Stop if uncommitted user changes would be overwritten or mixed into the optimization.
   - Fetch the latest base branch.
   - If the user supplies an existing PR URL, branch, or head SHA, continue from that head unless they explicitly request a fresh branch or PR.
   - Otherwise use a clean branch or worktree based on the fetched base.
   - Confirm the branch contains no unrelated commits or file changes.

2. Establish the target.
   - Record the exact local benchmark name/filter, the Callgrind instruction metric, and the requested threshold.
   - Default to Valgrind instruction count, where lower is better. Do not silently compare wall time, percentages from a cloud report, or a differently named stage.
   - Record the Cargo benchmark target (`benches` or `rspack_sources`) and the narrowest filter that includes the stage.
   - Record the requested optimization scope, PR URL if any, branch, base SHA, head SHA, and current round number.
   - If the user supplies only a cloud benchmark label, run a local discovery pass and map it to one exact local benchmark filter before optimizing. Never reuse the cloud percentage or absolute value.

3. Freeze the local environment.
   - Build the image once and record its image ID from `environment.txt`.
   - Use Docker's native platform: `linux/arm64` on Apple Silicon and Arm hosts, or `linux/amd64` on x86-64 hosts. Valgrind must not run through cross-architecture QEMU emulation.
   - Do not compare absolute instruction counts across architectures. Base and candidate must use the same native platform and image ID.
   - Prepare fixtures once. Use the same absolute fixture directory for every comparison and do not refresh it mid-goal.
   - Keep the benchmark target, filter, tool versions, `GLIBC_TUNABLES`, allocator settings, and measurement mode unchanged.
   - If any frozen input changes, invalidate all prior comparisons and remeasure the base plus the latest retained round.

4. Create an immutable base measurement.
   - Create or reuse a clean worktree at the PR base SHA or fetched base branch.
   - Measure the target through the helper before implementing an optimization.
   - Save results outside both Git worktrees, for example under a goal-specific directory in `/tmp`.
   - Read the exact stage value from each `run-*.instructions`, confirm each corresponding log ran the intended filter, and apply the repetition rule above.
   - Record the base SHA, median value, individual values, result directory, image ID, and fixture directory.
   - Never replace the original base with a later candidate. Remeasure it only when frozen inputs were invalidated.

5. Create or identify the fixed progress report.
   - When a PR exists, maintain one persistent PR comment for every round and pushed commit. Reuse an existing log instead of creating duplicates.
   - For work without a PR, maintain the same table in the task updates and final report.
   - Create a fresh PR after the first scoped change is committed and pushed if the task requires a PR.

## Per-Round Checklist

Run these steps in order for every round.

1. Modify code locally.
   - Choose the next likely bottleneck from the local Valgrind stage result, retained-round history, profiling artifacts, or code inspection.
   - Implement the smallest scoped change that plausibly improves the exact target stage.
   - Avoid unrelated cleanup, formatting churn, snapshot updates, and generated output unless required.
   - If the prior round found a correctness or review issue, fix it before adding another optimization idea.

2. Verify correctness locally.
   - Run targeted tests covering the changed behavior.
   - Run the relevant lint/check command for the touched language, including Rust check or clippy coverage for Rust changes.
   - Run format verification rather than broad formatting churn.
   - Follow repository build order: build JS before JS tests, build the Rust binding before Rust tests, and use the full dev build for mixed changes.
   - Treat known unrelated local watcher or SWC flakiness as reportable context, not silent success.
   - Fix any related failure and repeat validation before continuing.
   - Do not treat a faster Valgrind result as correctness evidence.

3. Run a correctness-focused independent review.
   - Review the current diff independently from the implementation pass before committing.
   - Use a subagent only when it is available and explicitly allowed; otherwise perform a fresh local diff review.
   - Focus on behavior preservation, ordering, deduplication, hashing/equality, diagnostics, concurrency, cancellation, and cache lifetime where relevant.
   - Fix valid concerns and repeat correctness validation plus review. Record the technical reason when a concern is invalid.

4. Commit the candidate snapshot.
   - Review the diff for unrelated edits.
   - Commit with a focused title, usually `perf: ...`.
   - Do not push yet. Measure a committed SHA so every result maps to immutable source.

5. Measure the candidate locally.
   - Run the helper against the candidate checkout with the frozen fixture directory, benchmark target, filter, image, platform, and environment.
   - Require the exact target stage in every repetition and retain all raw logs.
   - Use the median under the repetition rule.
   - Compute improvement for a lower-is-better instruction count as `(comparison - candidate) / comparison * 100`.
   - Compare the first candidate with the immutable base. Compare every later candidate with the most recent retained snapshot, including a retained correctness-only fix, because that snapshot is the candidate's actual code parent.
   - Also compute the candidate's total improvement from the immutable base for the goal threshold.
   - If another stage may have regressed, measure it separately with its own exact filter. Do not combine multiple stages into one instruction count.

6. Decide whether to retain the round.
   - Retain a performance round only when it improves on its comparison point and passes correctness/review validation.
   - If it is neutral or slower, revert the candidate with a new follow-up commit. Do not erase the failed attempt by amending or resetting history.
   - Keep a correctness-only fix when validation improves even if the performance result is neutral or slower. Record its measured value as the comparison point for the next candidate.
   - If the target is met, stop performance iteration after reporting and handling currently available review comments.
   - If the result improves but remains below target and rounds remain, choose the next scoped direction immediately.
   - If the round limit is reached, stop after reverting a non-improving final round and report the best retained result.

7. Push and report the decided state.
   - Push the retained candidate or the follow-up revert commit.
   - Create the PR if required and it does not yet exist.
   - Request or re-request code review when the repository workflow supports it.
   - Update the fixed progress report with the candidate commit, retained/reverted decision, exact local values, delta, correctness result, and result directory.
   - Do not start a CI timer, wait for GitHub Actions, or read CodSpeed feedback.

8. Handle available code review comments.
   - Inspect unresolved human and automated review threads that are currently available before declaring the workflow terminal and at the start of a resumed round.
   - Do not wait or poll for future comments as part of the measurement loop.
   - If a live comment is reasonable, implement the fix, reply with what changed, resolve the thread, and start a new locally validated round.
   - If it is not reasonable, reply with the technical reason and supporting evidence, resolve the thread, and continue without changing code.
   - Inspect outdated threads against current code before resolving them.
   - Do not report completion while a current requested-change thread remains unresolved.

## Measurement Interpretation

- Use only the exact integer from each local `run-*.instructions` file. It is the sum of Callgrind `totals:` values produced while the selected benchmark boundary was instrumented.
- Use individual repeated values plus their median; never average together different stage names, benchmark filters, fixtures, image IDs, or source SHAs.
- Treat a missing stage, mismatched filter, missing/zero instruction count, Valgrind error, or changed environment as invalid rather than as a regression.
- If base and candidate repetitions are inconsistent, first check that the same image and fixture directory were used. Rebuild neither source nor fixture selectively.
- A performance claim must name the base SHA and value, candidate SHA and value, percentage delta, image ID, benchmark target/filter, exact stage, and result paths.
- The local result determines the optimization goal. CI results may be mentioned as unrelated repository status only when the user explicitly asks for them.

## Progress Report Format

Keep one fixed PR comment, or the equivalent task report when there is no PR:

```markdown
## Local Valgrind Optimization Log

Target: `<exact measured stage>` >= `<threshold>` improvement
Scope: `<requested scope>`
Environment: `<image id>`, `<native Docker platform>`, `<benchmark target> <filter>`
Base: `<sha>` at `<instruction count>`
Current status: `<running|passed|failed|blocked>`
Latest head: `<sha>`

| Round | Commit  | Change      | Local Valgrind               | Decision              | Correctness |
| ----- | ------- | ----------- | ---------------------------- | --------------------- | ----------- |
| 1     | `<sha>` | `<summary>` | `<base -> candidate; delta>` | `<retained/reverted>` | `<summary>` |
```

Update this report rather than duplicating it. Keep entries concise and include links or paths to raw local logs.

## Correctness Guardrails

- When caching derived state, verify it remains stable for the full pass where it is reused.
- When adding a fast path, compare it with the original path in every relevant mode.
- When changing data structures, check ordering, deduplication, hashing, equality, and user-visible diagnostics.
- When changing concurrency or scheduling, check determinism, cancellation, shared-state ownership, and cleanup.
- When changing generated files, verify source and generated output remain consistent.
- Fix or technically dismiss every correctness concern from local commands or review before retaining the round.

## Reporting

Keep updates short and decision-oriented:

- State the PR URL when present, branch, head SHA, round, exact stage, threshold, and scope.
- State the code change and why it should affect the measured stage.
- List local correctness commands, ignored unrelated flakiness, and independent review result.
- State the Docker image ID, platform, fixture directory, benchmark target/filter, result paths, repeated values, median, base delta, and retained-round delta.
- State whether the round was retained or reverted and whether the fixed progress report was updated.
- State the status of currently available review comments and how each was handled.
- State explicitly that CI and CodSpeed cloud feedback were not used or monitored.
- If more work is needed, name the next optimization or correctness direction.
