---
name: rspack-sftrace
description: Use sftrace, which is based on LLVM Xray instrumentation, to trace all Rust function calls. This can be used for performance analysis and troubleshooting.
---

# Rspack Sftrace

## Overview

Use sftrace (LLVM XRay) to trace rspack's Rust function calls and convert them to perfetto protobuf format for performance analysis and troubleshooting.

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
# Resolve binding path from the target project (example: examples/react)
BINDING_NODE="$(pnpm -C examples/react exec node -p 'require.resolve("@rspack/binding-linux-x64-gnu")')"

# Regex mode
sftrace filter -p "$BINDING_NODE" -r 'finish_modules|FlagDependencyExportsPlugin' -o sftrace.filter

# List mode (one regex per line)
# sftrace filter -p "$BINDING_NODE" --list symbols.list -o sftrace.filter
```

If your platform package name differs, replace `@rspack/binding-linux-x64-gnu` accordingly.

### 4) Record sftrace (example: build in `examples/react`)

When `-f` points to a file outside your current directory, prefer an absolute path.

```sh
# Full trace
sftrace record -o sf.log -- pnpm -C examples/react build

# Filtered trace
sftrace record -f sftrace.filter -o sf.filtered.log -- pnpm -C examples/react build
```

### 5) Analyze sf.log

Convert sftrace log to perfetto protobuf format.

```sh
sftrace convert sf.filtered.log -o sf.filtered.pb.gz
```

### 6) Optional: Visualization using [viztracer](https://github.com/gaogaotiantian/viztracer)

```sh
vizviewer --use_external_processor sf.filtered.pb.gz
```

Use this only for visualization.

## Filtering Notes

- `sftrace filter` matches function symbols by regex/list. It is not a first-class crate-path/module-path filter.
- Filtering does not automatically keep all descendants. If a child function symbol does not match your filter, it may disappear from the trace.
- Cross-thread relationships (for example via rayon) are not reconstructed as a single uninterrupted call chain.
- For complete call stacks, record without filter (or with a broad filter) and narrow down during analysis.
