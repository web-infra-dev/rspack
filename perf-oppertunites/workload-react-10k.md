# Workload: build-tools-performance / react-10k

This document records the exact steps used to run the `react-10k` workload with
local Rspack builds for profiling.

## Repository Setup

```bash
# clone outside the rspack workspace
git -c http.lowSpeedLimit=1 -c http.lowSpeedTime=600 \
  clone https://github.com/rstackjs/build-tools-performance.git \
  /home/ubuntu/build-tools-performance

pnpm -C /home/ubuntu/build-tools-performance install

# link local rspack packages
pnpm -C /home/ubuntu/build-tools-performance add -w \
  @rspack/core@link:/workspace/packages/rspack \
  @rspack/cli@link:/workspace/packages/rspack-cli
```

## Local Rspack Build

```bash
pnpm install
pnpm run build:js
pnpm run build:binding:profiling
```

## Perf Permissions (Linux)

```bash
echo 0 | sudo tee /proc/sys/kernel/kptr_restrict
echo 1 | sudo tee /proc/sys/kernel/perf_event_paranoid
```

## Profiling Command

The system kernel (`6.1.147`) did not provide a matching perf package. We used
the perf binary from `linux-tools-6.8.0-100` directly:

```bash
cd /home/ubuntu/build-tools-performance/cases/react-10k

/usr/lib/linux-tools-6.8.0-100/perf record -o ./perf.data \
  -e cycles:uk -F 4000 --call-graph dwarf -- \
  node --perf-prof --perf-basic-prof --interpreted-frames-native-stack \
  /workspace/packages/rspack-cli/bin/rspack.js \
  -c ./rspack.config.mjs
```

The build completed in ~2.8s and emitted asset size warnings (expected for this
workload).

## Reporting

```bash
/usr/lib/linux-tools-6.8.0-100/perf report -i ./perf.data \
  --stdio --no-children -g none --percent-limit 0.5
```

Call-graph reporting (`-g graph`) repeatedly timed out due to missing build-id
debug entries, so the flat report was used for analysis.
