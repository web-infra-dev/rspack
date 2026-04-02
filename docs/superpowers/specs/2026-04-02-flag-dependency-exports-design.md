# Flag Dependency Exports Two-Phase Reexport Design

Date: 2026-04-02
Repo: `rspack`
Branch: `codex/flag-dependency-exports-topo`
Worktree: `/Users/bytedance/.config/superpowers/worktrees/rspack/flag-dependency-exports-topo`

## Summary

Redesign `FlagDependencyExportsPlugin` so dependency export collection and reexport propagation are separated into two phases:

1. Collect all dependency `get_exports` results in parallel.
2. Apply local export mutations first.
3. Defer all non-nested reexport resolution until a topology-driven propagation phase.
4. Resolve reexports by traversing SCC-compressed topology from leaf SCCs back toward roots, using a local fixed-point loop inside each SCC.

The goal is to reduce repeated recursive backtracking from `get_target` during `finish_modules`, while preserving current semantics for nested export structures and interop-sensitive cases.

## Context

Today `FlagDependencyExportsPlugin` performs these steps in one feedback loop:

1. Collect `ExportsSpec` from a batch of modules in parallel.
2. Merge collected specs into `ExportsInfoArtifact`.
3. Recompute target exports info for reexports while merging.
4. If a target changed, enqueue dependent modules into the next batch.

This means reexport-heavy graphs pay for repeated backtracking during the same pass. The hot path is not only export collection, but the repeated invalidation and re-entry caused by reexport target resolution.

## Scope

This design intentionally covers only the first iteration of the redesign.

Included:

- Ordinary local dependency exports.
- Non-nested reexports.
- A reusable analysis artifact recovered in `FINISH_MODULES`.
- SCC-based reexport propagation.
- Full performance and correctness verification.

Excluded from the new deferred propagation path:

- Nested export propagation semantics.
- JSON nested export trees.
- Fake namespace object and CommonJS interop semantics when they require structured local export creation.

These excluded cases are still collected in phase 1, but they are applied as local structured mutations before deferred reexport propagation begins.

## Goals

- Reduce backtracking cost in `FlagDependencyExportsPlugin`.
- Keep collection highly parallel.
- Replace repeated recursive invalidation with ordered propagation on a prebuilt topology.
- Make the intermediate graph reusable by incremental `FINISH_MODULES`.
- Preserve existing semantics for nested and interop-sensitive cases.

## Non-Goals

- Unifying every export shape into a single propagation engine in the first patch.
- Rewriting `ExportsInfoArtifact`.
- Optimizing every incremental graph update path on day one.
- Changing public rspack behavior or export semantics.

## Design Overview

The redesign splits work into two major phases inside `finish_modules`.

### Phase 1: Parallel Collection

For each affected module, walk dependencies and call dependency-specific collection logic in parallel.

Each dependency result is normalized into one of two categories:

- `LocalCollected`
  - Export information that can be determined during collection.
  - This includes both flat local exports and structured local exports.
- `DeferredReexport`
  - Reexport information that records topology and mapping metadata only.
  - No target backtracking or `get_target`-style recursive resolution happens in this phase.

Important rule:

- Reexport-producing dependencies may be updated so their collection output carries deferred reexport metadata instead of immediately forcing target resolution through the current merge path.

### Phase 2: Ordered Apply And Propagation

After collection:

1. Apply all `LocalCollected` mutations.
2. Build or update reexport topology from all `DeferredReexport` items.
3. Compress the reexport graph into SCCs.
4. Process SCCs from leaves toward roots.
5. Inside each SCC, repeatedly apply deferred reexports until the SCC reaches a fixed point.
6. Once an SCC stabilizes, propagate its effect to incoming SCCs.

This changes the propagation model from recursive batch backtracking to topology-guided convergence.

## Artifact Design

Add a new recoverable artifact on `Compilation`:

- Name: `DependencyExportsAnalysisArtifact`
- Incremental pass: `IncrementalPasses::FINISH_MODULES`

This artifact does not replace `ExportsInfoArtifact`.

Responsibilities of `DependencyExportsAnalysisArtifact`:

- Store normalized collection results for modules participating in finish-modules export analysis.
- Store reexport graph edges and reverse edges.
- Cache SCC membership and topological order.
- Track dirty modules and affected SCCs during incremental rebuilds.

Responsibilities of `ExportsInfoArtifact` remain unchanged:

- Store final provided export state.
- Store final target export info wiring.
- Remain the source of truth consumed by later stages.

## Data Model

### Per-Module Analysis

Each analyzed module stores:

- `local_apply: Vec<LocalExportOp>`
- `deferred_reexports: Vec<DeferredReexportOp>`
- `outgoing_reexport_targets: IdentifierSet`
- `incoming_dependents: IdentifierSet`
- `dirty: bool`

### Local Export Operations

`LocalExportOp` has two forms:

- `Flat`
  - A flat local export mutation that can be applied with the existing fast path idea of clone-merge-commit.
- `Structured`
  - A structured local export mutation that may allocate nested `ExportsInfo`, set `exports_info_owned`, or encode fake namespace / interop-aware local structure.

Both belong to `LocalCollected`, because both are determinable during collection.

Only the apply strategy differs:

- `Flat` may be committed in parallel per-module.
- `Structured` is committed sequentially.

Neither category participates in deferred reexport propagation.

### Deferred Reexport Operations

Each deferred reexport stores:

- owner module
- target module
- dependency id
- exposed export name
- target export path or namespace marker
- visibility flags such as `hidden`
- `priority`
- `can_mangle`
- `terminal_binding`
- exclusion or hidden sets for star-like cases

The deferred representation records all information required to later resolve exports without recursively exploring the graph during collection.

### Topology Cache

The artifact also caches:

- module to SCC id mapping
- SCC node membership
- SCC outgoing and incoming edges
- SCC topological order
- SCC propagation waves for parallel scheduling

## Dependency Collection Changes

Reexport-producing dependencies are allowed to change their collection behavior.

Instead of treating reexports as immediately merged `ExportsSpec` that trigger backtracking, they should produce deferred reexport metadata.

This primarily affects dependencies whose current `get_exports` result encodes:

- direct reexports
- namespace reexports
- star reexports
- reexport forms that are non-nested but today still cause target chasing

Structured local cases remain local:

- nested exports stay as structured local mutations
- fake namespace object shapes stay as structured local mutations
- CommonJS interop-sensitive synthetic local structure stays as structured local mutations

This keeps the first version focused: defer graph propagation only when the semantic dependency is really another module's export state.

## Propagation Algorithm

### Step 1: Apply Local Collected Mutations

Before any reexport propagation:

1. Reset affected modules' provide info as today.
2. Apply all flat local mutations.
3. Apply all structured local mutations.

After this step, `ExportsInfoArtifact` already contains a stable base layer of all locally-known export structure.

### Step 2: Build Reexport Topology

Construct a directed graph:

- edge direction: `owner module -> target module`

This represents "owner module's provided exports depend on target module's provided exports".

### Step 3: SCC Compression

Compress the graph into SCCs.

Why:

- acyclic regions should converge in one leaf-to-root sweep
- cyclic reexport regions need local iteration but should not force the whole graph into repeated batch backtracking

### Step 4: Leaf-To-Root Propagation

Process SCCs in reverse topological order, but not as one global serial queue.

Instead, partition SCCs into propagation waves:

- leaf SCCs first
- root-like SCCs last
- SCCs in the same wave have no parent-child dependency relationship with each other

Execution rule:

- SCCs in the same propagation wave may run in parallel
- ordering is required only across waves, not within a wave

For each SCC:

1. Read already-stabilized target state from outgoing SCCs.
2. Resolve all deferred reexports owned by modules in the SCC.
3. Apply changes to the owner modules' export target/provided state.
4. Repeat until the SCC no longer changes.

### Step 5: Fixed Point Inside SCC

Inside a cyclic SCC:

- iterate only modules in that SCC
- stop when no owner module in the SCC changes

This yields a local fixed-point loop instead of global recursive backtracking.

The important concurrency boundary is:

- parallel across independent SCCs in the same wave
- sequential only where topology requires parent-after-child ordering
- iterative only inside a single SCC until that SCC converges

## Incremental Strategy

The artifact is recovered in `before_finish_modules`.

### Dirty Rules

Hard-dirty modules:

- modules returned by `mutations.get_affected_modules_with_module_graph(...)`
- modules missing an analysis entry
- modules whose dependency collection output changed
- modules whose outgoing reexport edge set changed

Propagation-dirty SCCs:

- SCCs containing any hard-dirty module
- all SCCs that can reach those SCCs through reverse reexport edges

Dirty propagation execution still follows propagation waves:

- only the wave closure covering propagation-dirty SCCs needs to rerun
- within each rerun wave, independent SCCs may still execute in parallel

### First-Version Rebuild Strategy

To keep the first implementation correct and bounded:

- recollect only hard-dirty modules
- fully replace their stored local/deferred analysis entry
- rebuild SCC/topology cache when any dirty module's edge set changed
- rerun propagation only on the closure of propagation-dirty SCCs

This is intentionally less ambitious than incremental SCC surgery, but it already avoids a full reexport recomputation when only part of the graph changes.

## Correctness Constraints

The redesign must preserve:

- provided export visibility
- target export mapping
- priority handling
- hidden export handling
- terminal binding semantics
- namespace-style reexport behavior
- behavior of structured local interop cases

Fallback policy for edge cases:

- if a dependency cannot be stably represented as `LocalCollected` or `DeferredReexport`, keep it on the existing direct-processing path
- if an SCC-local propagation path encounters an unsupported shape, fall back only for that local region rather than disabling the whole redesign

## Risks

### Risk 1: Structured Local Mutations Accidentally Deferred

If fake namespace or nested export structure is mistakenly deferred like a plain reexport, semantics can drift because those cases create local nested export structure, not just cross-module dependency edges.

Mitigation:

- classify these explicitly as `Structured`
- keep their apply path outside deferred propagation

### Risk 2: Topology Cache Reuse With Stale Edges

Incremental recovery can become incorrect if edge sets are reused after a dependency collection shape changed.

Mitigation:

- treat any edge-set delta as topology-dirty
- rebuild SCC/topology cache whenever topology changed

### Risk 3: SCC Fixed-Point Cost In Large Cycles

Large reexport cycles may still be expensive.

Mitigation:

- confine iteration to SCC-local scope
- preserve current local-region fallback if a pathological cycle appears

## Implementation Outline

1. Add `DependencyExportsAnalysisArtifact` to `rspack_core`.
2. Recover it in `before_finish_modules`.
3. Extend `Compilation` to carry the artifact.
4. Refactor `FlagDependencyExportsPlugin` collection into a normalize-and-store stage.
5. Introduce `LocalCollected` and `DeferredReexport` normalization.
6. Update reexport-producing dependency collection logic to emit deferred metadata.
7. Keep nested/interop-sensitive shapes as structured local mutations.
8. Implement SCC/topology build.
9. Implement SCC leaf-to-root wave scheduling with local fixed-point.
10. Wire dirty module and propagation-dirty SCC invalidation.
11. Add focused regression coverage for reexport graphs, cycles, star reexports, namespace reexports, and structured local cases.
12. Run full performance and correctness validation.

## Verification Plan

### Worktree And Branch

- Create a new worktree under `~/.config/superpowers/worktrees`
- Use a fresh branch from latest `origin/main`
- Perform implementation in subagent-driven mode

### Performance Baseline

On latest `origin/main`, collect baseline performance using the same style as `.github/workflows/bench-rust.yml`:

1. `pnpm run bench:prepare`
2. `cargo codspeed build -m simulation --profile codspeed -p rspack_benchmark --features codspeed`
3. `RAYON_NUM_THREADS=1 pnpm run bench:rust`

### Performance Validation

After the implementation is complete, rerun the same CodSpeed flow on the feature branch and compare against the baseline from latest `origin/main`.

Acceptance:

- overall CodSpeed performance must be better than the baseline

### Functional Validation

Before tests:

1. `pnpm run build:cli:dev`

Then run:

1. `pnpm run test:rs`
2. `pnpm run test:unit`

### Preflight Validation

Required checks:

1. `cargo fmt --all --check`
2. `cargo lint`

## Open Decisions Already Settled

The following decisions are fixed for implementation:

- first iteration covers ordinary local exports and non-nested reexports
- nested and interop-sensitive shapes stay local, not deferred
- reexports are collected only in parallel and resolved later during propagation
- SCC compression is required
- SCC-local fixed-point is required
- a reusable recoverable artifact is required
- performance acceptance is based on overall CodSpeed results versus latest `origin/main`

## Recommended Commit Strategy

Keep the implementation in small commits:

1. artifact and plumbing
2. collection normalization
3. SCC propagation
4. tests
5. performance-only cleanup if needed

This keeps rollback and benchmark comparison simpler while the algorithm changes are still settling.
