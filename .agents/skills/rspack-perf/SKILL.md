---
name: rspack-perf
description: Use when optimizing performance for user-specified files, features, compilation stages, Rust crates, JavaScript plugins, graph processing, parser work, chunking, code generation, or memory/CPU hot paths in Rspack.
---

# Rspack Performance Optimization

## Core Principle

Optimize from the shape of Rspack's data. Always consider the rough cardinality of internal structures before changing an algorithm:

`dependency/export_info` > `module/exports_info/module_graph_module` > `chunk` > `chunk_group` > `entry/runtime`.

The larger the structure, the more dangerous full scans, repeated traversals, broad cloning, eager materialization, and per-item allocation become. Avoid whole-graph or whole-compilation work on high-cardinality structures unless profiling proves it is necessary.

## First Pass

1. Identify the target feature, file, or compilation stage and the dominant data structures it touches.
2. Estimate whether the hot path scales with dependencies, exports, modules, chunks, chunk groups, entries, or runtimes.
3. Look for repeated CPU work and repeated allocation before designing a larger refactor.
4. Prefer small, measurable changes that preserve observable output.
5. If adding parallelism, respect Rspack's concurrency model: use `rayon` for CPU-bound synchronous work, use `rspack_parallel` abstractions for async orchestration, and avoid mixing rayon and tokio pools inside one workflow without a clear boundary.

## CPU Optimization Techniques

### Avoid repeated computation

- Cache stable results such as hashes, identifiers, dependency lists, export targets, side-effect states, sorted orders, and rule-match summaries.
- Move repeated query-time computation into one collection/preparation phase.
- Use per-compilation artifacts or transient caches for values that are only valid during one compilation.
- Memoize pure or effectively pure calculations when the key is cheap and the result is reused.

### Avoid repeated traversal

- Do not scan all dependencies, exports, modules, or graph connections inside another large loop.
- Collect the needed data once, then process it in batches.
- Maintain affected sets for incremental or mutation-driven work.
- Add reverse indexes or lookup tables when callers repeatedly search from broad collections.
- Convert recursive or backtracking graph propagation into a work queue that avoids revisiting the same node.

### Add fast paths for common cases

- Return early for empty, single-item, non-nested, unchanged, no-connection, or no-runtime cases.
- In rule and regex matching, recognize simple string, suffix, literal, and native-regex cases before invoking the full matcher.
- In parser and plugin flows, skip hook dispatch, tag lookup, and dependency bookkeeping when no matching hook/tag/dependency is present.
- In export and graph logic, bypass expensive target resolution for empty or simple non-nested exports.

### Defer expensive work

- Delay collection, sorting, flattening, string rendering, diagnostic construction, and detailed bailout reason generation until the result is actually needed.
- Store compact references, ranges, ids, or lightweight summaries instead of eagerly materializing full chains or trees.
- Avoid preparing detailed stats/debug information on the default build path unless it will be consumed.

### Batch state updates

- Split workflows into "collect changes" and "apply changes" phases.
- Batch runtime requirement updates, chunk updates, export propagation, hash updates, and graph patches.
- Avoid interleaving repeated mutable graph writes with repeated graph reads when a two-phase flow can use immutable data first.

### Reduce algorithmic complexity

- Replace broad candidate scans with indexes, buckets, or prefiltered candidate sets.
- Bucket by stable dimensions such as chunk length, runtime, cache group, module group, or dependency type.
- Prune impossible combinations before generating them.
- Avoid generating all combinations when an iterator, bounded search, or affected-only search is enough.

### Reduce dispatch and scheduling overhead

- Keep synchronous CPU hot paths synchronous.
- Avoid boxed futures, unnecessary trait-object dispatch, and deeply layered closures in tight loops.
- Prefer direct function calls or static dispatch when the call site is hot and the implementation set is known.
- Parallelize only after isolating read-only inputs and local outputs; merge results in a controlled final phase to avoid lock contention.

## Memory Optimization Techniques

### Reduce string churn

- Avoid repeated `format!`, `.to_string()`, and owned-string construction for identifiers, requests, resource names, runtime helpers, export names, and module ids.
- Prefer `&str`, interned strings, `Atom`, `Arc<str>`, or cached rendered strings where ownership is not required.
- Push final string rendering closer to output generation.
- Do not allocate an owned string only to perform a lookup.

### Reduce cloning and data movement

- Pass references for large structures.
- Share immutable data with `Arc` when ownership must cross phases.
- Clone the small field that is needed, not the whole module/dependency/export/chunk structure.
- Pass ids or keys across phases when the receiver can look up the full object.
- Remove unused metadata from hot structs.

### Reduce temporary containers

- Avoid creating `Vec`, `HashMap`, `HashSet`, `IndexMap`, or combination lists inside high-cardinality loops.
- Preallocate capacity when cardinality is known or cheaply estimated.
- Reuse scratch buffers when safe.
- Prefer streaming/iterator processing over `collect()` followed by one traversal.
- Do not use ordered maps/sets unless stable order is required.

### Choose data structures for the key shape

- Use specialized identifier, ukey, atom, or fixed-bitset collections when the key domain supports it.
- Consider `FxHashMap`/`FxHashSet` or identity hashing for interned or already-hashed keys.
- Prefer `Vec` plus stable indexes for dense key spaces.
- Use compact small-vector storage for common short lists or paths.

### Avoid eager materialization

- Do not build full member chains, export paths, module-reference paths, sorted module lists, or chunk combinations until a consumer needs them.
- Represent common cases compactly.
- Use visitors or iterators when the caller can consume items without owning a full container.

### Keep hot structures small

- Move cold stats, diagnostics, debug strings, and optional metadata out of hot structs.
- Store detailed data in side maps keyed by id when only a minority of items need it.
- Prefer compact summaries between stages instead of copying full objects.

## Validation Requirements

Functional validation:

- Run the relevant focused tests when available.
- Run `pnpm run build:cli:dev` before `pnpm run test:unit`.
- Always complete `pnpm run test:unit` after the optimization.

Performance validation:

- Validate with CodSpeed cases.
- Prefer the CodSpeed `compilation stages` case that directly covers the optimized feature.
- If no matching compilation-stage case exists, use the closest existing CodSpeed case for the affected feature and explain the coverage gap.
- If adding or changing benchmark coverage is necessary, keep the case focused on the optimized stage and avoid unrelated noise such as minimization unless it is the target.

Before submitting:

- Run `pnpm run format:rs`.
- Run `pnpm run format:js`.
- Run `cargo clippy --workspace --all-targets --all-features`.
- Summarize the hot path, the data cardinality risk, the chosen CPU/memory optimization technique, functional test result, CodSpeed performance result, format result, and clippy result.

## PR and Benchmark Follow-up

After the optimization is complete, ask the user whether they want to create a PR.

If the user wants a PR:

1. Switch to a branch whose name starts with `perf/`.
2. Commit the optimization with a conventional performance commit, usually `perf(<scope>): <summary>` or `perf: <summary>`.
3. Push the branch and create a PR whose title starts with `perf: `.
4. Write the PR body around:
   - what changed
   - why the change should improve performance
   - which hot path and data cardinality risk it addresses
   - the expected CPU and/or allocation benefit
5. Do not include a `Validation` section in the PR body.
6. After the PR exists, automatically trigger the Ecosystem Benchmark workflow for the PR:

```sh
gh workflow run 'Ecosystem Benchmark' -f pr=<pr-number>
gh run list --workflow 'Ecosystem Benchmark' --limit 5
```

Use the PR number as the workflow input. A run on `main` does not benchmark the PR branch.

If the user allows waiting for GitHub CI:

1. Wait for the relevant GitHub checks and the Ecosystem Benchmark run to finish.
2. Fetch the PR comments and locate the CodSpeed performance report.
3. Compare improvements and regressions in the report.
4. If the report shows regressions, analyze likely causes from the changed hot path, data structure cardinality, CPU work, and allocation behavior.
5. Perform one additional optimization iteration when there is a plausible fix, then update the PR branch, push again, and rerun the Ecosystem Benchmark workflow.

## Common Mistakes

- Optimizing around chunks or runtimes while ignoring that dependencies and export infos may be orders of magnitude more numerous.
- Adding a cache without defining its lifetime or invalidation boundary.
- Making a hot structure larger to save one rare computation.
- Adding parallelism before removing repeated serial work.
- Trading a CPU bottleneck for large temporary allocations.
- Verifying only output snapshots without checking the relevant CodSpeed case.
