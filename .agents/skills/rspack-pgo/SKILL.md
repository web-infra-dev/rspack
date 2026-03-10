---
name: rspack-pgo
description: >-
  Run Rspack's perf-guided optimization loop for `cases/all` and similar
  workloads: create an isolated worktree, build a profiling binding, benchmark
  with `RSPACK_BINDING`, collect and compare `perf` hotspots, implement small
  Rust changes, validate, commit, push, and trigger the Ecosystem Benchmark
  workflow after each pushed commit. Use this when the goal is iterative
  performance work, not just one-off profiling.
---

# Rspack PGO

This skill is for a perf-guided optimization loop, not Rust/LLVM `-C profile-generate/-C profile-use`.

Use this when the task is "keep finding the next hotspot and improve it". For raw `perf` capture details or `samply` import, also read [`../rspack-perf-profiling/SKILL.md`](../rspack-perf-profiling/SKILL.md).

## Default setup

- Keep the user's main worktree untouched. Use a clean temp worktree for experiments.
- Default benchmark repo: `https://github.com/hardfist/rspack-bench-repo`
- Default case: `<bench-repo>/cases/all`
- Test the temp worktree's native binding through `RSPACK_BINDING`; do not rely on the installed `@rspack/binding`.
- Prefer small, isolated commits. Keep only changes that improve end-to-end time.

## Loop

### 1) Create an isolated branch

Prefer a temp worktree based on `origin/main`:

```sh
git fetch origin
git worktree add /tmp/rspack-perf-$(date +%Y%m%d) -b <branch-name> origin/main
```

If the active branch already has user changes, do not reuse it for perf experiments.

### 2) Build a profiling binding

```sh
pnpm run build:binding:profiling
```

The binding used for local benchmark runs should be:

```sh
<worktree>/crates/node_binding/rspack.linux-x64-gnu.node
```

### 3) Measure a local baseline

Run the benchmark from the benchmark repo, but point it at the temp worktree binding:

If the benchmark repo is not present yet, clone it to a sibling directory or another scratch location first:

```sh
git clone https://github.com/hardfist/rspack-bench-repo ../rspack-bench-repo
cd ../rspack-bench-repo/cases/all
```

```sh
env RSPACK_BINDING=<worktree>/crates/node_binding/rspack.linux-x64-gnu.node \
  /usr/bin/time -f '%e real %U user %S sys' \
  ./node_modules/.bin/rspack -c ./rspack.config.js
```

For `cases/all`, compare against a recent valid baseline built the same way. One run is enough to reject obviously bad ideas; use 2-3 runs before keeping a change.

### 4) Capture a flat `perf` profile first

Start with a flat `cycles:u` profile. On this repository the `.node` binary is large enough that DWARF callgraphs can become too expensive.

```sh
perf record -o /tmp/rspack-cases-all.perf.data \
  -e cycles:u -F 2500 -- \
  env RSPACK_BINDING=<worktree>/crates/node_binding/rspack.linux-x64-gnu.node \
  ./node_modules/.bin/rspack -c ./rspack.config.js

perf report -i /tmp/rspack-cases-all.perf.data \
  --stdio --no-children -g none --percent-limit 0.2 | head -n 160
```

Use DWARF callgraphs only when flat symbols are insufficient.

### 5) Pick the next hotspot pragmatically

Prioritize changes that can move wall time, not just symbol percentages.

Good candidates in `cases/all` have been:

- export analysis: `ExportsInfoGetter::prefetch`, `get_used_name`
- module graph/cache lookups: `OverlayMap::get`, `GetExportsTypeCache::get`
- tasking overhead: channel `start_send`, `Stealer::steal`, tiny parallel tasks
- allocation churn: `mi_free`, repeated `String` or `ArcPath` creation, `reserve_rehash`

Treat low-single-digit hash micro-hotspots carefully. Keep them only if the end-to-end benchmark moves.

### 6) Implement the smallest plausible fix

Common winning patterns:

- replace generic maps/sets with project-specific containers such as `IdentifierMap`, `IdentifierSet`, `UkeyMap`, `UkeySet`
- split composite keys like `(Identifier, bool)` when one dimension is tiny
- add `with_capacity` / `reserve` in hot temporary collections
- remove per-item cross-thread sends in hot loops; prefer parallel `collect` plus sequential apply
- avoid materializing `String` when an `Atom`, `Identifier`, or borrowed string is enough
- reduce repeated graph lookups by caching within the local loop, not by adding a large global cache first

### 7) Validate before deciding

Minimum validation for Rust-side perf changes:

```sh
cargo fmt --all --check
cargo check -p <crate>
pnpm run build:binding:profiling
```

Run targeted tests if the touched area already has coverage. Run broader checks only when they are relevant and not blocked by known unrelated failures.

### 8) Keep or revert based on end-to-end results

Keep a change only if at least one of these is true:

- local `cases/all` compile time improves
- a previously top-level hot symbol clearly disappears and wall time does not regress

Revert experiments that only reshuffle percentages without improving compile time.

### 9) Commit, push, and trigger benchmark immediately

After every kept change:

```sh
git commit -am "refactor(<scope>): <perf change>"
git push
gh workflow run 'Ecosystem Benchmark' -f pr=<pr-number>
gh run list --workflow 'Ecosystem Benchmark' --limit 5
```

The benchmark workflow input is defined in [`.github/workflows/ecosystem-benchmark.yml`](../../../.github/workflows/ecosystem-benchmark.yml). Use `pr=<pr-number>`; a run on `main` does not benchmark the PR branch.

If `git push` is unavailable because of local network or DNS issues, use `gh`/GitHub API as a fallback to update the PR branch, then trigger the workflow. In that fallback mode, local `git status` may stay dirty because local `HEAD` does not know about the remote-only commit.

### 10) Rebase and keep history clean

Before opening a new PR or after long-running iteration:

```sh
git fetch origin
git rebase origin/main
```

If the branch was updated through the GitHub API instead of local git, sync the local branch before assuming `git status` represents unpushed work.

## Reporting

When reporting progress, include:

- kept commit SHA
- PR URL
- latest local benchmark numbers
- `perf.data` path for the kept profile
- workflow run URL after each pushed commit
- the specific hot symbols that went down or disappeared

## Common traps

- A lower `perf` percentage is not enough; keep changes only when compile time improves or stays flat with a clearly better hotspot profile.
- `gh run list` may show an in-progress `Ecosystem Benchmark` on `main`; that does not replace a `workflow_dispatch` run for the PR.
- If `git status` shows a modified tracked file after a fallback remote update, compare the local file blob hash with the remote contents API before assuming something is uncommitted.
- Do not churn the user's main worktree with benchmark artifacts. Keep `perf.data`, scratch logs, and benchmark-only files under `/tmp` or the temp worktree.
