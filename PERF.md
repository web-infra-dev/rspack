# Rspack Line-by-Line Performance Review

This report summarizes a line-by-line CPU sampling review of Rspack using the
existing `tests/bench/fixtures/ts-react` benchmark fixture. The analysis is
based on the generated `line-report.txt` from the profiling script.

## Method

- Benchmark fixture: `tests/bench/fixtures/ts-react` (via
  `scripts/profile/bench-ts-react.config.cjs`)
- Command:
  ```sh
  pnpm run profile:line-report -- \
    --outDir ./.rspack-profile-ts-react \
    --traceFilter OVERVIEW \
    --perf /usr/lib/linux-tools-6.8.0-100/perf \
    --addr2line /usr/bin/llvm-addr2line
  ```
- Sample size: 22 CPU samples (each line item in the report represents ~4.55%)
- Build: `build:binding:profiling` + `build:js`

> **Note:** Because the run is short, the sample count is low and each entry
> has equal weight. For higher confidence, rerun with `--repeat` and/or a higher
> sample rate.

## Biggest Performance Impacts (Line-by-Line Review)

### 1. SWC minifier + usage analyzer work (largest Rust-side hotspot group)

The most frequent Rust-side samples sit inside SWC minifier + usage analyzer
paths, indicating that minification and analysis dominate the CPU time for this
fixture:

- `swc_ecma_minifier/src/compress/optimize/conditionals.rs:501`
- `swc_ecma_minifier/src/compress/optimize/sequences.rs:728`
- `swc_ecma_minifier/src/compress/optimize/...` (multiple optimizer passes)
- `swc_ecma_usage_analyzer` in `triomphe/src/thin_arc.rs:248`
- `swc_ecma_visit` generated visitors (traversal overhead)

**Impact:** SWC minification and analysis passes are a primary CPU sink for the
bench build. This is expected in production builds, but it is also the most
material target for optimization and caching.

**Opportunities**
- Cache minify results for unchanged modules or cache SWC AST analysis outputs.
- Consider pruning or gating costly minifier passes for small modules.
- Reuse analyzer state where possible to reduce traversal overhead.

### 2. HashMap-heavy paths (allocation / rehash pressure)

HashMap activity appears repeatedly:

- `swc_ecma_codegen/src/lib.rs:625` (HashMap remove in codegen)
- `swc_ecma_transforms_base/src/fixer.rs:919` (HashMap insert)
- `crates/rspack_plugin_javascript/src/dependency/esm/esm_import_specifier_dependency.rs:362`
  (HashMap insert for import specifier tracking)

**Impact:** heavy map churn can degrade overall throughput, especially when
combined with repeated AST traversal. These hot lines suggest rehash/resize
and insertion overhead at scale.

**Opportunities**
- Pre-size HashMaps in hot paths.
- Reuse maps across passes where possible.
- Replace HashMap updates with immutable or append-only structures in
  performance-critical loops.

### 3. Parser + AST traversal in Rspack plugin code

The report shows Rspack plugin parser work and AST traversal:

- `crates/rspack_plugin_javascript/.../esm_import_specifier_dependency.rs:362`
  (tracking import specifiers)
- `swc_ecma_ast` and `swc_ecma_visit` traversal sites used by parser plugins

**Impact:** repeated AST walks and dependency extraction are non-trivial, and
they compound with minification. This is a typical cost center in bundlers.

**Opportunities**
- Reduce duplicate AST traversals during dependency scans.
- Cache derived dependency metadata when the module is unchanged.

### 4. Tracing / registry overhead

There is a hotspot in tracing subscriber storage:

- `sharded-slab/src/shard.rs:282` (registry access)

**Impact:** tracing data structure access can be non-trivial when tracing is
enabled. This suggests profiling with tracing disabled for more stable results,
or limiting tracing filters to reduce overhead when not needed.

**Opportunities**
- Narrow `RSPACK_PROFILE` filters when profiling specific subsystems.
- Avoid tracing in hot loops when not necessary.

### 5. JS/V8 + libc overhead

The report also includes runtime and libc entries:

- `__tls_get_addr`, `pthread_mutex_lock`, and other libc routines
- V8 internals (string handling, shared function info creation, line ends)

**Impact:** These are runtime overheads from Node/V8 and system libraries. They
are expected but can mask smaller Rust hotspots in short runs.

**Opportunities**
- Increase `--repeat` to dampen VM warmup costs.
- Re-run with a larger fixture or a longer build to surface Rust hot paths.

## Summary of Biggest Impacts

1. **SWC minifier + usage analyzer** — dominant CPU time on the Rust side.
2. **HashMap insert/remove churn** — visible in SWC and Rspack plugin code.
3. **Parser + AST traversal** — repeated scans for dependencies and transforms.
4. **Tracing registry + runtime costs** — measurable overhead when tracing.

## Next Steps

- Re-run with `--repeat 5` (or higher) to gather more samples and stabilize
  the line-level distribution.
- Consider isolating minifier-only runs to separate parser/codegen costs from
  minification costs.
- Use the generated `rspack.pftrace` for timeline analysis of compilation
  phases and correlate with the line-level hotspots above.
