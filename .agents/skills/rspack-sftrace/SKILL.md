---
name: rspack-sftrace
description: Use sftrace, which is based on LLVM Xray instrumentation, to trace all Rust function calls. This can be used for performance analysis and troubleshooting.
---

# Rspack Sftrace

## Overview

Use sftrace to trace rspack's Rust function calls and convert them to perfetto protobuf format for performance analysis and troubleshooting.

## Workflow

### 1) Build sftrace tools

```sh
git clone https://github.com/quininer/sftrace
cd sftrace
cargo build --release
mkdir "$(./target/release/sftrace record --print-solib-install-dir)"
cp ./target/release/libsftrace.so "$(./target/release/sftrace record --print-solib-install-dir)/"
```

### 2) Build sftrace-enabled binding (once per code change)

```sh
SFTRACE=1 pnpm run build:binding:debug
```

### 3) Record sftrace (examples/basic)

```sh
cd examples/basic/
sftrace record -- pnpm build
```

### 4) Analyze sf.log

Convert sftrace log to perfetto protobuf format.

```sh
sftrace convert sf.log -o sf.pb.gz
```

### 5) Optional: Visualization using [viztracer](https://github.com/gaogaotiantian/viztracer)

```sh
vizviewer --use_external_processor sf.pb.gz
```

Use this only for visualization.
