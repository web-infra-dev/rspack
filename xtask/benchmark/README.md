# Rspack Rust benchmarks

Rust benchmark cases live in `xtask/benchmark/cases` and `xtask/benchmark/stages`.
The `rspack_sources` benchmarks live in their own `rspack_sources` benchmark
target so CodSpeed builds and runs them in a separate binary, isolated from the
allocator state left behind by the larger compilation benchmark suite.
Walltime-only bundle benchmarks follow the same pattern and live in the
separate `walltime` benchmark target.

## Prepare benchmark fixtures

Some benchmark cases use fixtures from `.bench/rspack-benchcases`. Prepare them before running the full benchmark suite or any bundle-related case:

```bash
pnpm run bench:prepare
```

The prepare step also creates a local `threejs-10x` fixture by copying the
upstream `threejs/src` input ten times. This larger input is registered only by
the isolated `walltime` benchmark target, so regular simulation runs keep using
the smaller default benchmark set. The `threejs-10x` walltime benchmark writes
outputs through the native filesystem instead of the in-memory filesystem used
by the regular bundle benchmarks. Rust benchmark concurrency is selected with
`BENCH_MODE=simulation|walltime` (default: `simulation`). Walltime bundle
benchmarks use `BENCH_MODE=walltime`, which caps machine-size-dependent
parallelism at up to 16 threads for each of Tokio workers, Tokio blocking tasks,
and Rayon.

## Run in CI mode

The CI command runs `cargo codspeed run` directly:

```bash
pnpm run bench:rust
```

When this command is run outside a CodSpeed runner environment, `cargo-codspeed` may only check benchmarks and print `Checked:` entries instead of making CPU simulation measurements.

## Build benchmark binaries

Use the build helper to run the same CodSpeed build command used by CI:

```bash
pnpm run build:bench
```

The script expands to:

```bash
cargo codspeed build -m simulation --profile codspeed -p rspack_benchmark --bench benches --bench rspack_sources
```

This only builds the benchmark binaries for CodSpeed simulation mode. It does not execute measurements. Both benchmark targets are selected in a single `cargo codspeed build` invocation so the later `cargo codspeed run` step can collect both benchmark suites.

To build only the isolated `rspack_sources` target:

```bash
pnpm run build:bench:rspack-sources
```

## Run local CPU simulation measurements

Use the local helper script to run benchmarks through the CodSpeed runner in CPU simulation mode:

```bash
pnpm run bench:rust:local
```

Pass a benchmark name filter after `--` to run a single benchmark:

```bash
pnpm run bench:rust:local -- build_chunk_graph
```

To run only the isolated `rspack_sources` binary:

```bash
pnpm run bench:rust:local -- --bench rspack_sources
```

The script expands to:

```bash
mkdir -p /tmp/rspack-codspeed-valgrind-tmp && TMPDIR=/tmp/rspack-codspeed-valgrind-tmp BENCH_MODE=simulation RSPACK_BENCHCASES_DIR=$PWD/.bench/rspack-benchcases codspeed run -m simulation -- cargo codspeed run -m simulation
```

This command:

- Creates a stable temporary directory for CodSpeed and Valgrind files
- Sets `TMPDIR` so the temporary files are written to that directory
- Sets `BENCH_MODE=simulation` to match the single-threaded CI simulation environment
- Sets `RSPACK_BENCHCASES_DIR` to the prepared benchmark fixtures
- Wraps `cargo codspeed run -m simulation` in `codspeed run -m simulation` so local runs use the CodSpeed runner environment instead of only checking benchmarks

A real local simulation run prints `Measured:` entries. The script sets `TMPDIR` to `/tmp/rspack-codspeed-valgrind-tmp`, so CodSpeed and Valgrind temporary files are easy to inspect:

```bash
find /tmp/rspack-codspeed-valgrind-tmp -maxdepth 3 -type f
```

Typical files include `profile.*.out/valgrind.log`, `profile.*.out/runner.log`, and `vgdb-pipe-*` files.

## Requirements

Local CPU simulation requires:

- `cargo-codspeed`
- `codspeed`
- CodSpeed's Valgrind build

Check the installed tools with:

```bash
cargo codspeed --version
codspeed --version
valgrind --version
```
