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

### Line-by-line report (perf)

For a line-level CPU report on Linux, use the built-in profiling script. It wraps `perf record` and emits a line-by-line report (`line-report.txt`) plus a `rspack.pftrace` file for timeline analysis.

```sh
# Run a profiling build first (or add --build to the script).
pnpm run build:binding:profiling
pnpm run build:js

# Generate the report using the default bench fixture (ts-react)
pnpm run profile:line-report
```

You can customize the inputs and output location:

```sh
pnpm run profile:line-report -- \
  --config scripts/profile/bench-ts-react.config.cjs \
  --outDir ./.rspack-profile-ts-react \
  --rate 199 \
  --traceFilter OVERVIEW
```

Notes:

- The script requires `perf` (install via `linux-tools-common` on Ubuntu).
- Use `--build` to run `pnpm run build:binding:profiling` and `pnpm run build:js` automatically.
- Extra arguments after `--` are forwarded to `rspack build`.
- Use `--repeat N` to run the build multiple times for more samples.

### Rsdoctor timeline

If we want to analyze the time cost of loaders and plugins or the compilation behavior of loaders, we can use Rsdoctor to view:

![image](https://assets.rspack.rs/others/assets/rsdoctor/rsdoctor-loader-timeline.png)

Refer to [Rsdoctor Compilation Analysis](/guide/optimization/profile#use-rsdoctor)

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
