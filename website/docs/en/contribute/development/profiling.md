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

## Memory profiling

Memory profilers observe allocations at the allocator boundary. The regular Rspack release package uses mimalloc, while `@rspack-debug/core` uses the system allocator. Use `@rspack-debug/core` when collecting Heaptrack data or dynamically injecting jemalloc.

In the project you want to profile, override `@rspack/core` with the same version of `@rspack-debug/core` as described in [Debugging](/contribute/development/debugging#using-rspack-debugcore), then reinstall the dependencies. For example, if the project uses `@rspack/core@2.1.0` with pnpm:

```json title="package.json"
{
  "pnpm": {
    "overrides": {
      "@rspack/core": "npm:@rspack-debug/core@2.1.0"
    },
    "peerDependencyRules": {
      "allowAny": ["@rspack/*"]
    }
  }
}
```

```sh
pnpm install
```

| Profiler           | Output   | Analyzer                           | Best used for                                      |
| ------------------ | -------- | ---------------------------------- | -------------------------------------------------- |
| Heaptrack          | `*.gz`   | `heaptrack_print`, `heaptrack_gui` | Allocation call stacks and temporary allocations   |
| jemalloc profiling | `*.heap` | `jeprof`                           | Live memory and cumulative allocation flame graphs |

:::warning
Heaptrack and jemalloc use different data formats. `jeprof` cannot read a Heaptrack data file. Do not enable Heaptrack and jemalloc profiling in the same run.
:::

### Heaptrack

[Heaptrack](https://github.com/KDE/heaptrack) intercepts system allocator calls and records allocation call stacks. Install it using your Linux distribution's package manager, then run the build under Heaptrack:

```sh
heaptrack --record-only -o ./rspack-heaptrack \
  node ./node_modules/@rspack/cli/bin/rspack.js build
```

Inspect the generated file from the terminal or GUI:

```sh
heaptrack_print ./rspack-heaptrack.gz | less
heaptrack_gui ./rspack-heaptrack.gz
```

The exact output filename is printed when Heaptrack exits. `--record-only` prevents Heaptrack from trying to open the GUI automatically, which is useful in WSL, containers, and remote shells.

The system allocator used by `@rspack-debug/core` allows Heaptrack to capture Rspack and SWC allocations. The regular release package uses mimalloc and bypasses the system allocator hooks, so its Rust allocation data is incomplete. Depending on the Heaptrack version, the GUI may show Rust v0 symbol names beginning with `_R` instead of demangled names; this affects display only, not the recorded stacks. For a demangled text report, install [`rustfilt`](https://github.com/luser/rustfilt) and pipe the output through it:

```sh
heaptrack_print ./rspack-heaptrack.gz | rustfilt | less
```

### jemalloc profiling

On Linux, a build that uses the system allocator can be redirected to a profiling-enabled shared jemalloc with `LD_PRELOAD`. Install jemalloc, `jeprof`, and Graphviz using your distribution's packages. On Debian or Ubuntu, the shared library is provided by `libjemalloc2`; the development package commonly provides the profiling tools.

For example, on Debian or Ubuntu:

```sh
sudo apt install heaptrack libjemalloc-dev graphviz
```

Locate the installed library and collect profiles:

```sh
mkdir -p /tmp/rspack-jemalloc
JEMALLOC=$(ldconfig -p | awk '/libjemalloc.so.2/{print $NF; exit}')

MALLOC_CONF='prof:true,prof_active:true,prof_final:true,lg_prof_sample:19,lg_prof_interval:26,prof_prefix:/tmp/rspack-jemalloc/rspack' \
  LD_PRELOAD="$JEMALLOC" \
  node ./node_modules/@rspack/cli/bin/rspack.js build
```

The important options are:

- `prof:true` enables profiling.
- `prof_active:true` starts sampling immediately.
- `prof_final:true` writes a final dump when the process exits.
- `lg_prof_sample:19` samples approximately every 512 KiB of allocations.
- `lg_prof_interval:26` writes a dump after approximately every 64 MiB of allocation activity.
- `prof_prefix` controls where profile files are written.

Find the native Rspack binding loaded by the project:

```sh
RSPACK_BINDING=$(node -e 'require("@rspack/core"); const binding = Object.keys(require.cache).find(file => /rspack\..+\.node$/.test(file)); if (!binding) throw new Error("Rspack native binding not found"); process.stdout.write(binding)')
PROFILE=$(ls -t /tmp/rspack-jemalloc/rspack.*.heap | head -n 1)
```

Generate an SVG for a selected dump and open it in a browser:

```sh
jeprof --show_bytes --functions --svg \
  "$RSPACK_BINDING" \
  "$PROFILE" \
  > rspack-memory.svg
```

For a text report sorted by cumulative memory:

```sh
jeprof --show_bytes --functions --text --cum \
  "$RSPACK_BINDING" \
  "$PROFILE"
```

The default `inuse_space` report describes memory that was live when the dump was written. To investigate allocation traffic, add `prof_accum:true` to `MALLOC_CONF` and pass `--alloc_space` to `jeprof`. Cumulative allocated bytes are not peak memory.

jemalloc only reports allocations routed through jemalloc. Node.js, V8, native libraries, memory mappings, and profiler metadata can also contribute to process RSS. Compare the profile with `/usr/bin/time -v` when investigating total or peak process memory.

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
