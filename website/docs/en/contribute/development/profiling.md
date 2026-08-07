---
description: "In this section, we'll explore how to profile Rspack for identifying bottlenecks"
---

# Profiling

In this section, we'll explore how to profile Rspack for identifying bottlenecks.
By examining where Rspack spends its time, we can gain insights into how to improve performance.
Since different profilers have different strengths. It is good to use more than one.

<!-- toc -->

## Build release version with debug info

Performance analysis should be conducted on a release version that includes debug information. This approach ensures accurate performance results while providing sufficient debug information for analysis. Use the following command to profiling using local build rspack.

1. Build a release version with debug information:

```sh
pnpm build:binding:profiling
```

2. Change `@rspack/core` and `@rspack/cli` to use `link` protocol to link to local build Rspack:

```diff title="package.json"
  dependencies: {
-    "@rspack/core": "x.y.z",
-    "@rspack/cli": "x.y.z",
     # link protocol only works in pnpm
+    "@rspack/core": "link:{your_rspack_repo}/packages/rspack",
+    "@rspack/cli": "link:{your_rspack_repo}/packages/rspack-cli"
  }
```

3. Reinstall:

```sh
pnpm install
```

## Memory profiling with jemalloc

The `@rspack-debug/core` package uses jemalloc with heap profiling support by default on supported native platforms. Recording is disabled until it is enabled through `_RJEM_MALLOC_CONF`, so the debug package can still be used normally without producing profile files.

:::note
jemalloc profiling is available in the debug package on Linux and macOS. Windows MSVC and Wasm packages continue to use the default allocator because jemalloc profiling is not supported on those targets. The regular `@rspack/core` package is not affected.
:::

### Install the debug package

Override `@rspack/core` with `@rspack-debug/core` in the project whose build you want to profile. For example, with pnpm:

```json title="package.json"
{
  "pnpm": {
    "overrides": {
      "@rspack/core": "npm:@rspack-debug/core@latest"
    },
    "peerDependencyRules": {
      "allowAny": ["@rspack/*"]
    }
  }
}
```

Reinstall the project's dependencies after adding the override:

```sh
pnpm install
```

When investigating a version-specific issue, replace `latest` with the same version as the project's original `@rspack/core` dependency. Remove the override after profiling.

### Collect heap profiles

Create an output directory, then run the project's normal build command with jemalloc profiling enabled. For a project that directly uses the Rspack CLI:

```sh
mkdir -p /tmp/rspack-jemalloc

cd /path/to/project
_RJEM_MALLOC_CONF='prof:true,prof_active:true,prof_final:true,lg_prof_sample:19,lg_prof_interval:26,prof_prefix:/tmp/rspack-jemalloc/rspack' \
pnpm exec rspack build
```

If the project uses Rsbuild or another Rspack-based tool, keep the same environment variable and run that tool's normal build command instead.

`tikv-jemallocator` prefixes its runtime configuration variable, so use `_RJEM_MALLOC_CONF` rather than `MALLOC_CONF`.

The options above have the following effects:

- `prof:true` enables heap profiling. It must be set when the process starts.
- `prof_active:true` starts sampling immediately.
- `prof_final:true` writes a final `.f.heap` file when the process exits.
- `lg_prof_sample:19` samples approximately once per 512 KiB of allocations. Decrease it for more detail at the cost of additional overhead.
- `lg_prof_interval:26` writes an interval `.i.heap` file after approximately every 64 MiB of allocation activity.
- `prof_prefix` controls where profile files are written.

Add `prof_accum:true` when cumulative allocation data is required. It increases profiler memory usage, so omit it when only live allocations are needed. Avoid `prof_gdump:true` for normal builds because a small increase in the high-water mark can produce many dumps and significantly perturb the measurement.

### Generate function allocation graphs

Install `jeprof` from jemalloc 5.x and Graphviz. Resolve the native binding loaded by `@rspack-debug/core`, then pass that `.node` file to `jeprof` as the program containing the Rust symbols:

```sh
RSPACK_BINDING=$(node -e 'require("@rspack/core"); const binding = Object.keys(require.cache).find(file => /rspack\..+\.node$/.test(file)); if (!binding) throw new Error("Rspack native binding not found"); process.stdout.write(binding)')

jeprof --show_bytes --functions --exclude='_+rjem_' --svg \
  "$RSPACK_BINDING" \
  /tmp/rspack-jemalloc/rspack.<pid>.<sequence>.heap \
  > rspack-memory.svg
```

Generate a cumulative text report for easier hotspot comparison:

```sh
jeprof --show_bytes --functions --exclude='_+rjem_' --text --cum \
  "$RSPACK_BINDING" \
  /tmp/rspack-jemalloc/rspack.<pid>.<sequence>.heap
```

When `prof_accum:true` was enabled, add `--alloc_space` to inspect all sampled allocations made during the build:

```sh
jeprof --alloc_space --show_bytes --functions --exclude='_+rjem_' --svg \
  "$RSPACK_BINDING" \
  /tmp/rspack-jemalloc/rspack.<pid>.<sequence>.heap \
  > rspack-alloc-space.svg
```

The default `inuse_space` view shows allocations that were live at the time of the dump. `alloc_space` shows cumulative allocation traffic and is useful for finding temporary allocation churn; it is not peak memory usage. jemalloc covers the Rust global allocator in the Rspack binding, while Node.js, V8, native libraries, memory mappings, and profiler metadata also contribute to process RSS. Use `/usr/bin/time -l` on macOS or `/usr/bin/time -v` on Linux when a whole-process peak RSS measurement is also required.

## CPU profiling

### Samply

[Samply](https://github.com/mstange/samply) supports performance analysis for both Rust and JavaScript simultaneously. Follow these steps to perform a complete performance analysis:

- Run the following command to start performance analysis:

```sh
samply record -- node --perf-prof --perf-basic-prof --interpreted-frames-native-stack {your_rspack_folder}/rspack-cli/bin/rspack.js -c {your project}/rspack.config.js
```

- After the command execution, the analysis results will automatically open in the [Firefox Profiler](https://profiler.firefox.com/). The screenshot below is from a [Samply profiler](https://profiler.firefox.com/public/5fkasm1wcddddas3amgys3eg6sbp70n82q6gn1g/calltree/?globalTrackOrder=0&symbolServer=http%3A%2F%2F127.0.0.1%3A3000%2F2fjyrylqc9ifil3s7ppsmbwm6lfd3p9gddnqgx1&thread=2&v=10).

:::warning
Node.js currently only supports `--perf-prof` on Linux platforms. JavaScript profiling in Samply depends on `--perf-prof` support. If you need to use Samply for JavaScript profiling on other platforms, consider using Docker for profiling, or you can compile Node.js yourself for macOS using [node-perf-maps](https://github.com/tmm1/node/tree/v8-perf-maps) for profiling purposes.
:::

#### JavaScript profiling

Rspack’s JavaScript typically runs in the Node.js thread. Select the Node.js thread to view the time distribution on the Node.js side.

![Javascript Profiling](https://assets.rspack.rs/rspack/assets/profiling-javascript.png)

#### Rust profiling

Rspack’s Rust code usually runs in the tokio thread. Select the tokio thread to view the time distribution on the Rust side.

![Rust Profiling](https://assets.rspack.rs/rspack/assets/profiling-rust.png)

### Rsdoctor timeline

If we want to analyze the time cost of loaders and plugins or the compilation behavior of loaders, we can use Rsdoctor to view:

![image](https://assets.rspack.rs/others/assets/rsdoctor/rsdoctor-loader-timeline.png)

Refer to [Rsdoctor Compilation Analysis](/guide/optimization/use-rsdoctor)

## Mac Xcode instruments

Xcode instruments can be used to produce a CPU profile if you are on a Mac.

![image](https://github.com/SyMind/rspack-dev-guide/assets/19852293/124e3aee-944a-4509-bb93-1c9213f026d3)

To install Xcode Instruments, simply install the Command Line Tools:

```bash
xcode-select --install
```

For normal Rust builds, [`cargo instruments`](https://github.com/cmyr/cargo-instruments) can be used as the glue
for profiling and creating the trace file.

Since Rspack takes quite a while to build, you can use the following procedure without invoking `cargo instruments`.
It has the same effect.

In workspace root's `Cargo.toml`, turn on debug symbols and disable symbol stripping in the `[profile.release]` section

```toml
[profile.release]
debug = 1 # debug info with line tables only
strip = false # do not strip symbols
```

Then build the project

```bash
pnpm run build:cli:release
```

The final binary is located at `packages/rspack-cli/bin/rspack` once the project is built.

Under the hood, `cargo instruments` invokes the `xcrun` command,
which means we can run the following in our own project that uses Rspack.

```bash
xcrun xctrace record --template 'Time Profile' --output . --launch -- /path/to/rspack/packages/rspack-cli/bin/rspack build
```

It produces the following output

```
Starting recording with the Time Profiler template. Launching process: rspack.
Ctrl-C to stop the recording
Target app exited, ending recording...
Recording completed. Saving output file...
Output file saved as: Launch_rspack_2023-04-24_11.32.06_9CFE3A63.trace
```

We can open the trace file by

```bash
open Launch_rspack_2023-04-24_11.32.06_9CFE3A63.trace
```
