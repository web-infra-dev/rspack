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

## Tracing

[`tracing`](https://crates.io/crates/tracing) is used to instrumenting Rspack.

The supported tracing levels for

- release builds are `INFO`, `WARN` and `ERROR`
- debug builds are `TRACE`, `DEBUG`, `INFO`, `WARN` and `ERROR`

Two ways to enable tracing:

- if you are using `@rspack/cli`, you can enable it by `RSPACK_PROFILE` environment variable.
- if you are using `@rspack/core` without `@rspack/cli`, you can enable it by `rspack.experiments.globalTrace.register` and `rspack.experiments.globalTrace.cleanup`, checkout [how we implement `RSPACK_PROFILE` in `@rspack/cli` with these two function](https://github.com/web-infra-dev/rspack/blob/9be47217b5179186b0825ca79990ab2808aa1a0f/packages/rspack-cli/src/utils/profile.ts#L219-L224) for more details.

### Chrome

[`tracing-chrome`](https://crates.io/crates/tracing-chrome) is supported for viewing tracing information graphically.

![image](https://github.com/SyMind/rspack-dev-guide/assets/19852293/1af08ba1-a2e9-4e3e-99ab-87c1e62e067b)

Setting the environment variable `RSPACK_PROFILE=TRACE=layer=chrome` before running Rspack, for example

```bash
RSPACK_PROFILE=TRACE=layer=chrome rspack build
```

produces a trace file (`.rspack-profile-${timestamp}-${pid}/trace.json`) in the current working directory.

The JSON trace file can be viewed in either `chrome://tracing` or [ui.perfetto.dev](https://ui.perfetto.dev).

### Terminal

Granular tracing event values can be viewed inside the terminal via `RSPACK_PROFILE=TRACE=layer=logger`, for example

```bash
RSPACK_PROFILE=TRACE=layer=logger rspack build
```

will print the options passed to Rspack as well as each individual tracing event.

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

![Javascript Profiling](https://assets.rspack.dev/rspack/assets/profiling-javascript.png)

#### Rust profiling

Rspack’s Rust code usually runs in the tokio thread. Select the tokio thread to view the time distribution on the Rust side.

![Rust Profiling](https://assets.rspack.dev/rspack/assets/profiling-rust.png)

### Node.js profiling

If we find that the performance bottleneck is on the JS side (e.g. js loader), then we need to further analyse the js side, and we can use Nodejs Profiling to analyse. for example

```bash
node --cpu-prof {rspack_bin_path} -c rspack.config.js
```

or

```bash
RSPACK_PROFILE=JSCPU rspack build
```

this will generates a cpu profile like `CPU.20230522.154658.14577.0.001.cpuprofile`, and we can use speedscope to visualize the profile, for example

```bash
npm install -g speedscope
speedscope CPU.20230522.154658.14577.0.001.cpuprofile
```

### Rsdoctor timeline

If we want to analyze the time cost of loaders and plugins or the compilation behavior of loaders, we can use Rsdoctor to view:

![image](https://assets.rspack.dev/others/assets/rsdoctor/rsdoctor-loader-timeline.png)

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
