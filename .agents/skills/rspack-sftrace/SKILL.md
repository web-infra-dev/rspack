---
name: rspack-sftrace
description: Use sftrace, which is based on LLVM Xray instrumentation, to trace all Rust function calls. This can be used for performance analysis and troubleshooting.
---

# Rspack Sftrace

## Overview

Use sftrace (LLVM XRay) to trace rspack's Rust function calls and convert them to perfetto protobuf format for performance analysis and troubleshooting.

Default workflow: run inside the target example directory (for example `examples/react`) and store all trace artifacts in that directory (not `/tmp`).

## Workflow

### 1) Build sftrace tools

```sh
git clone https://github.com/quininer/sftrace
cd sftrace
cargo build --release
mkdir "$(./target/release/sftrace record --print-solib-install-dir)"
cp ./target/release/libsftrace.so "$(./target/release/sftrace record --print-solib-install-dir)/"
```

### 2) Build sftrace-enabled profiling binding (once per code change)

```sh
SFTRACE=1 pnpm build:binding:profiling
```

### 3) Optional: Generate a filter file from symbols

`sftrace filter` works on function symbols from an object file (for rspack, the binding `.node` file).

```sh
# Enter the target example directory first
cd examples/react

# Prefer the locally built profiling binding from the monorepo
BINDING_NODE="$(realpath ../../crates/node_binding/rspack.linux-x64-gnu.node)"

# Regex mode
sftrace filter -p "$BINDING_NODE" -r 'finish_modules|FlagDependencyExportsPlugin' -o sftrace.filter

# List mode (one regex per line)
# sftrace filter -p "$BINDING_NODE" --list symbols.list -o sftrace.filter
```

If your binding file name differs by platform, replace the `.node` path accordingly.

### 4) Record sftrace (example: build in `examples/react`)

Run from the target example directory and keep outputs local to that example.

```sh
cd examples/react

TRACE_DIR="sftrace-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$TRACE_DIR"

# Full trace
sftrace record -o "$TRACE_DIR/sf.log" -- pnpm build

# Filtered trace (requires sftrace.filter from step 3)
sftrace record -f sftrace.filter -o "$TRACE_DIR/sf.filtered.log" -- pnpm build
```

### 5) Analyze sf.log

Convert sftrace log to perfetto protobuf format.

```sh
cd examples/react
TRACE_DIR="sftrace-YYYYMMDD-HHMMSS" # replace with your run directory
sftrace convert "$TRACE_DIR/sf.filtered.log" -o "$TRACE_DIR/sf.filtered.pb.gz"
```

### 6) Optional: Visualization using [viztracer](https://github.com/gaogaotiantian/viztracer)

```sh
vizviewer --use_external_processor "$TRACE_DIR/sf.filtered.pb.gz"
```

Use this only for visualization.

## Filtering Notes

- `sftrace filter` matches function symbols by regex/list. It is not a first-class crate-path/module-path filter.
- Filtering does not automatically keep all descendants. If a child function symbol does not match your filter, it may disappear from the trace.
- Cross-thread relationships (for example via rayon) are not reconstructed as a single uninterrupted call chain.
- For complete call stacks, record without filter (or with a broad filter) and narrow down during analysis.
