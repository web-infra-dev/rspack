# Native compiler with goexec

[中文说明](README.zh-CN.md)

The optional `rspack/goexec` Cargo feature runs native Rust compiler tasks on
published goexec 0.1.3. It enables the matching backend in `rspack_tasks`,
`rspack_fs`, and `rspack_plugin_schemes` together. Tokio remains the default.
Cargo.lock pins the dependency; no Git or local-path override is required.

## Integration

`rspack_tasks::runtime` selects `Runtime`, `JoinHandle`, `JoinError`, and spawn
operations. Compiler-context tasks and `rspack_parallel` joins use this shared
interface. Compile and run native compiler work inside the selected runtime;
the harness in `src/main.rs` shows the full compiler lifecycle.

Short native filesystem calls execute inside goexec's tracked blocking regions.
The resource reader retains the same synchronous filesystem implementation and
error context. CPU-heavy deferred destruction runs as ordinary executor work
inside the CPU permit budget. Rayon and Tokio task-local/synchronization
primitives remain in use. A dropped task handle still detaches the task.

This feature covers the native Rust Compiler API. Node/NAPI, JavaScript
plugins/loaders, watch timers, networking, and persistent-cache background
runtimes still require their existing runtime integration. Do not enable the
feature globally in Node bindings. The comparison harness disables persistent
cache. Cargo feature unification selects one backend for the dependency graph.

## Build and compare

Use the repository's pinned Rust toolchain, Python 3, and pnpm 10.11.0 for the
fixtures. From the repository root:

```sh
python3 xtask/goexec-compare/prepare-fixtures.py
cargo build --locked --profile executor-bench -p rspack_goexec_compare
cp target/executor-bench/rspack_goexec_compare target/goexec-compare/rspack-tokio
cargo build --locked --profile executor-bench -p rspack_goexec_compare --features goexec
cp target/executor-bench/rspack_goexec_compare target/goexec-compare/rspack-goexec

target/goexec-compare/rspack-tokio target/goexec-compare/fixtures threejs-10x development 12 2 3
target/goexec-compare/rspack-goexec target/goexec-compare/fixtures threejs-10x development 12 2 3
```

Arguments are `FIXTURE_ROOT PROJECT MODE WORKERS WARMUPS SAMPLES`.
Projects are `basic-react`, `threejs`, and `threejs-10x`. Modes are `development`,
`production-sourcemap` (minification off), and `production-minify` (source maps
off). Executor and Rayon worker counts match. Tokio permits 512 extra blocking
threads; goexec caps total workers at `WORKERS + 512`, plus its monitor, with a
100 microsecond handoff observation delay. The general runtime default is not
changed to the 12-worker setting used in the example.

The profile uses opt-level 3, LTO off, 16 codegen units, and mimalloc. The timer
covers `Compiler::run()`, including output writes with compare-before-emit off.
Each iteration creates a fresh Compiler with no persistent cache and a warm OS
filesystem cache. Construction, asset hashing, close/drop and runtime shutdown
are outside the primary timer. Both variants dispatch the root future to a
worker and wait 100 ms after cleanup. Every iteration checks compiler errors,
disk versus memory assets, per-asset SHA-256, and goexec shutdown completion.

For the macOS scheduling configuration used below, apply the same QoS shim to
both binaries:

```sh
clang -dynamiclib -O2 -Wall -Wextra -Werror xtask/goexec-compare/qos.c -o target/goexec-compare/bench-qos.dylib
DYLD_INSERT_LIBRARIES="$PWD/target/goexec-compare/bench-qos.dylib" target/goexec-compare/rspack-tokio target/goexec-compare/fixtures threejs-10x development 12 2 3
DYLD_INSERT_LIBRARIES="$PWD/target/goexec-compare/bench-qos.dylib" target/goexec-compare/rspack-goexec target/goexec-compare/fixtures threejs-10x development 12 2 3
```

The shim sets main and new pthreads to USER_INITIATED (25). Thread creation
includes its allocation and QoS-setting cost. It does not set CPU affinity or
isolate background load. Finish compilation before timing. Alternate executor
order across seven fresh process pairs, discard two warmups per process, and
retain three samples. Compare output hashes, module counts, and byte counts.
Store generated data under `target/` or the ignored `results/` directory.

## Prior validation

On 2026-09-23, Apple M3 Max (12P + 4E, 48 GiB), native Three.js-10x with 12
executor/Rayon workers showed these results. They were measured on upstream
Rspack `cbf189da225ee607d9f66197f0ad8e50d8258c57` with goexec main
`f5d82e14d8b068ecd0894065d3457bcf4ce33c54`. The published crate has identical
Rust sources. These archived timings are not a fresh performance claim for
later upstream revisions or this branch's repackaged integration.

| Mode | Tokio median | goexec median | Paired time change [95% CI] |
|---|---:|---:|---:|
| Development | 143.06 ms | 116.89 ms | -18.82% [-20.74, -17.11] |
| Source maps | 277.11 ms | 246.61 ms | -11.62% [-12.57, -10.63] |
| Minification | 1218.97 ms | 1187.06 ms | -2.89% [-3.85, -1.88] |

Changes are geometric means of seven paired process-block median ratios;
95% intervals use 10,000 paired bootstrap resamples. Each condition has 21
retained samples per variant. No outliers were removed and no multiple-comparison
adjustment was applied. Both binaries used the QoS shim and
`CARGO_PROFILE_EXECUTOR_BENCH_DEBUG=1` / `CARGO_PROFILE_EXECUTOR_BENCH_STRIP=none`.
Use those overrides for both build commands when reproducing this configuration.

All 630 builds at 1/12/16 workers and 18 separate three-project smoke builds
matched outputs, with successful shutdown and no spawn failures or capacity
delays. At 16 workers the time reduction was 2.0–3.3%; at one worker goexec was
3.5–10.8% slower. These shared-host native builds do not establish a universal
speedup or Node/NAPI, JS loader, watch/HMR, or persistent-cache performance.
