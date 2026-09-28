# Development commands

Run commands from the repository root unless shown otherwise. Command definitions live in [root package scripts](../package.json), [test package scripts](../tests/rspack-test/package.json), and [Cargo aliases](../.cargo/config.toml).

## Setup

Use the Rust toolchain in `rust-toolchain.toml`, the pnpm version in `package.json`, and the latest Node.js LTS.

`pnpm run setup` installs dependencies and builds the development binding and JS packages.

## Build

Build the changed code before running tests that consume it:

| Changed code | Build                        |
| ------------ | ---------------------------- |
| JavaScript   | `pnpm run build:js`          |
| Rust         | `pnpm run build:binding:dev` |
| Both         | `pnpm run build:cli:dev`     |

Other build variants:

- Native binding variants: `pnpm run build:binding:debug` or `pnpm run build:binding:release`.
- WASM: `pnpm run build:cli:dev:wasm`; browser: `pnpm run build:cli:dev:browser`.

## Tests

- Focused integration cases: `pnpm --dir tests/rspack-test run test -t "configCases/asset"`.
- Base integration suite: `pnpm --dir tests/rspack-test run test:base` (there is no root `test:base` script).
- JavaScript suites: `pnpm run test:unit`; Rust suites: `pnpm run test:rs`.
- HMR: `pnpm run test:hot`; E2E: `pnpm run test:e2e`; API types: `pnpm run test:type`.

See the [testing guide](../website/docs/en/contribute/development/testing.mdx) for harness details. New tests must follow the [repository's test restrictions](../AGENTS.md#adding-tests).

## Lint and formatting

- JavaScript lint: `pnpm run lint:js`.
- Rust check: `pnpm run lint:rs`; clippy: `cargo lint`.
- Rust format check: `cargo fmt --all --check`.
- Format Rust: `pnpm run format:rs`; JS/TS: `pnpm run format:js`.

The root lint and formatting commands cover the whole workspace.

## Performance and debugging

Rust benchmarks live in `xtask/benchmark/`. Build them with `pnpm run build:bench`, then run `pnpm run bench:ci` to prepare fixtures and execute benchmarks. `pnpm run bench:prepare` prepares fixtures separately.

Use `pnpm run build:binding:profiling` for a profiling binding. Tracing support lives in `crates/rspack_tracing/`.

To write an Allocative heap snapshot after each completed build or rebuild, compile the binding with `ALLOCATIVE=1` and set `RSPACK_ALLOCATIVE_DIR` for the Node process:

```sh
ALLOCATIVE=1 pnpm run build:binding:profiling
RSPACK_ALLOCATIVE_DIR=/tmp/rspack-allocative \
NODE_OPTIONS='--max-old-space-size=16384' \
pnpm --filter app-flow-chat run dev
```

The `.allocative` files include selected Rspack global roots, compilation assets and sources, ModuleGraph and module build info, ChunkGraph, code generation result sources, and runtime module sources. Common build-info payloads include JSON values, CSS metadata, loader assets, and filesystem snapshot indexes. The adjacent `*.coverage.json` lists covered roots and limitations. These are explicit reachable-object traversals, not complete heap scans. Plugin-specific module and dependency fields are counted shallowly; module source, its dependencies block, and common build-info fields are traversed. Map keys and asset names are omitted. Built-in `rspack_sources` types are traversed recursively; other `Source` implementations use the trait's shallow default unless they opt in. At most 10 snapshots are written per process by default; set `RSPACK_ALLOCATIVE_MAX_SNAPSHOTS` to lower that bound.

To write bounded process reports after each completed compilation, set `RSPACK_MEMORY_REPORT_DIR`:

```sh
RSPACK_MEMORY_REPORT_DIR=/tmp/rspack-memory/process \
NODE_OPTIONS='--max-old-space-size=16384' \
pnpm --filter app-flow-chat run dev
```

Each JSON report contains compilation duration, current RSS, a 100 ms sampled compilation peak RSS, process-lifetime peak RSS, V8 heap, `external`, and `arrayBuffers`. These process numbers include Node/V8 and native allocators such as Tailwind; they do not attribute those bytes to Rspack. The comparison gate uses process-lifetime peak RSS and default-allocator runs. The jemalloc profiling binding also fills `rust_live_bytes`; that build's RSS is not comparable to the default allocator's RSS.

Use the four report directories from three matched cold builds with the comparison gate. It requires at least three successful build samples per series and compares medians:

```sh
python3 scripts/memory/compare.py \
  --baseline-rust /tmp/rspack-memory/baseline/rust \
  --candidate-rust /tmp/rspack-memory/candidate/rust \
  --baseline-process /tmp/rspack-memory/baseline/process \
  --candidate-process /tmp/rspack-memory/candidate/process
```

The default gate requires at least 5% lower jemalloc live bytes and default-allocator peak RSS, with no more than 5% compilation-duration regression. Keep the app's source, command environment, and reports local.

The profiling binding uses jemalloc's sampled heap profiler and writes a snapshot after each completed build or rebuild when `RSPACK_JEMALLOC_PROFILE_DIR` is set. Start the Node process with `_RJEM_MALLOC_CONF=prof:true,prof_active:true,lg_prof_sample:19` to enable profiling. The default sampling interval is intentional for large projects; setting `lg_prof_sample:0` records every allocation and can add substantial overhead.

```sh
pnpm run build:binding:profiling
mkdir -p /tmp/rspack-heap
cd /path/to/flow-web-monorepo
RSPACK_JEMALLOC_PROFILE_DIR=/tmp/rspack-heap \
_RJEM_MALLOC_CONF='prof:true,prof_active:true,lg_prof_sample:19' \
NODE_OPTIONS='--max-old-space-size=16384' \
pnpm --filter app-flow-chat run dev
```

Each completed compilation writes a `*.heap` file. Analyze one with jemalloc's `jeprof` and the profiling binding as the symbol file:

```sh
cd /path/to/rspack
jeprof --show_bytes --inuse_space --text crates/node_binding/rspack.darwin-arm64.node /tmp/rspack-heap/<snapshot>.heap
jeprof --show_bytes --inuse_space --svg crates/node_binding/rspack.darwin-arm64.node /tmp/rspack-heap/<snapshot>.heap > /tmp/rspack-heap/heap.svg
```

On macOS, install `jeprof` with `brew install jemalloc` if it is not already available.

Use the binding artifact matching the host platform. The SVG is a call graph; `jeprof --collapsed` can be piped to Brendan Gregg's FlameGraph tool when a flame graph is preferred. These snapshots cover allocations routed through Rspack's Rust global allocator, not Node/V8 or independent native allocators.

For public CI fixtures, prepare and build the CodSpeed memory target, which covers a high-cardinality module graph and CSS/Tailwind development compilation:

```sh
pnpm run bench:prepare
pnpm run build:bench:memory
BENCH_MODE=memory \
RSPACK_BENCHCASES_DIR="$PWD/.bench/rspack-benchcases" \
cargo codspeed run --bench memory
BENCH_MODE=memory \
RSPACK_BENCHCASES_DIR="$PWD/.bench/rspack-benchcases" \
RSPACK_ALLOCATIVE_DIR=/tmp/rspack-allocative \
RSPACK_ALLOCATIVE_MAX_SNAPSHOTS=3 \
RSPACK_ALLOCATIVE_SNAPSHOT_MODULE_GRAPH=1 \
cargo bench --profile codspeed -p rspack_benchmark --bench memory --features rspack_benchmark/allocative -- --test
```

The reusable CI workflow runs CodSpeed memory mode on `ubuntu-24.04`. It uploads bounded Allocative summaries and coverage metadata only; app-flow-chat data is never part of the CI fixture. CodSpeed tracks allocations and peak memory during the benchmark, while Allocative describes the selected objects remaining after compilation.

See the [debugging guide](../website/docs/en/contribute/development/debugging.mdx) for VS Code configurations, JavaScript inspection, and `rust-lldb`. See [project layout](../website/docs/en/contribute/development/project.md) for core, plugin, API, CLI, and binding paths.
