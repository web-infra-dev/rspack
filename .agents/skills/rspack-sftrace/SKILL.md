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

### 5) Optional: Analyze sf.log by [polars](https://docs.pola.rs/)

Convert sftrace log to pola dataframe.

```sh
cd examples/react
TRACE_DIR="sftrace-YYYYMMDD-HHMMSS" # replace with your run directory
sftrace convert --type pola "$TRACE_DIR/sf.log" -o "$TRACE_DIR/sf.pola"
```

This will generate two files, whose schema format is as follows:

1. sf.pola

  This records all events from sftrace log.

  | name     | type        | description
  |----------|-------------|-------------
  | frame_id | uint64      | a unique id for each frame. a function's entry and exit have same frame id
  | parent   | uint64      | point to previous frame id. zero means non-existent
  | tid      | uint32      | thread id
  | func_id  | uint64      | function unique id
  | time     | nanoseconds | time elapsed since program started
  | kind     | uint32      | event type, 1 is entry, 2 is exit, 3 is tail call

2. sf.pola.symtab

  This records the function symbol name and file path of `func_id`.

  | name    | type   | description
  |---------|--------|-------------
  | func_id | uint64 | function unique id
  | name    | string | function symbol name (demangled)
  | path    | string | the file path and line number of function

You can use `python-polars` to perform data analysis on `sf.pola`.

```python
import polars as pl

sf = pl.scan_parquet("./sf.pola")
symtab = pl.scan_parquet("./sf.pola.symtab")

# Query the functions that appear most frequently
(
  sf
    .filter(pl.col("kind").eq(1))
    .group_by("func_id")
    .agg(pl.len().alias("func_count"))
    .top_k(10, by="func_count")
    .join(symtab, on="func_id")
    .collect()
)

# Query the leaf frame of longest execution time
(
  sf
    .filter(~pl.col("frame_id").is_in(pl.col("parent").implode()))
    .group_by("frame_id")
    .agg([
      pl.col("func_id").first(),
      pl.col("time").filter(pl.col("kind").eq(1)).first().alias("entry_time"),
      pl.col("time").filter(pl.col("kind").is_in([2, 3])).last().alias("exit_time"),
    ])
    .filter(pl.col("exit_time").is_not_null())
    .with_columns(pl.col("exit_time").sub("entry_time").alias("duration"))
    .top_k(10, by="duration")
    .join(symtab, on="func_id")
    .collect()
)
```

### 6) Optional: Visualization sf.log

Convert sftrace log to perfetto protobuf format.

```sh
cd examples/react
TRACE_DIR="sftrace-YYYYMMDD-HHMMSS" # replace with your run directory
sftrace convert "$TRACE_DIR/sf.log" -o "$TRACE_DIR/sf.pb.gz"
```

Visualization using [viztracer](https://github.com/gaogaotiantian/viztracer)

```sh
vizviewer --use_external_processor "$TRACE_DIR/sf.pb.gz"
```

Use this only for visualization.

## Filtering Notes

- `sftrace filter` matches function symbols by regex/list. It is not a first-class crate-path/module-path filter.
- Filtering does not automatically keep all descendants. If a child function symbol does not match your filter, it may disappear from the trace.
- Cross-thread relationships (for example via rayon) are not reconstructed as a single uninterrupted call chain.
- For complete call stacks, record without filter (or with a broad filter) and narrow down during analysis.
