# Profiling

在本节中，我们将探讨如何分析 Rspack Profile 以识别性能瓶颈。

通过检查 Rspack 将时间花在哪里，我们可以深入了解如何提高性能。

由于不同的分析器有不同的优势，使用多个分析器是一个好的选择。

<!-- toc -->

## Build release version with debug info

性能分析应基于包含调试信息的发布版本进行。这种方法既能确保性能结果的准确性，又能提供充足的调试信息用于分析。使用以下命令使用本地的 rspack 进行 profiling

1. 构建带有调试信息的发布版本：

```sh
just build release-debug
```

2. 更改 `@rspack/core` 和 `@rspack/cli`，使用 `link` 协议链接到本地​​构建的 Rspack：

```diff title="package.json"
  dependencies: {
-    "@rspack/core": "x.y.z",
-    "@rspack/cli": "x.y.z",
     # link protocol only works in pnpm
+    "@rspack/core": "link:{your_rspack_repo}/packages/rspack",
+    "@rspack/cli": "link:{your_rspack_repo}/packages/rspack-cli"
  }
```

3. 重新安装依赖：

```sh
pnpm install
```

## CPU profiling

### Samply

[Samply](https://github.com/mstange/samply) 支持同时对 Rust 和 JavaScript 进行性能分析，可通过如下步骤进行完整的性能分析:

- 运行以下命令启动性能分析：

```sh
samply record -- node --perf-prof --perf-basic-prof --interpreted-frames-native-stack {your_rspack_folder}/rspack-cli/bin/rspack.js -c {your project}/rspack.config.js
```

- 命令执行完毕后会自动在 [Firefox profiler](https://profiler.firefox.com/) 打开分析结果，如下截图来自 [Samply profiler](https://profiler.firefox.com/public/5fkasm1wcddddas3amgys3eg6sbp70n82q6gn1g/calltree/?globalTrackOrder=0&symbolServer=http%3A%2F%2F127.0.0.1%3A3000%2F2fjyrylqc9ifil3s7ppsmbwm6lfd3p9gddnqgx1&thread=2&v=10)。

:::warning
Node.js 目前仅在 Linux 平台支持 `--perf-prof`，而 Samply 里的 JavaScript Profiling 依赖 `--perf-prof`的支持，如果你需要在其他平台使用 Samply 进行 JavaScript Profiling，可以选择使用 docker 里进行 profiling，或者可以基于 [node-perf-maps](https://github.com/tmm1/node/tree/v8-perf-maps) 自行在 macOs 平台编译 Node.js 用于 profiling。
:::

#### JavaScript profiler

Rspack 的 JavaScript 代码通常执行在 Node.js 线程里，选择 Node.js 线程查看 Node.js 侧的耗时分布。

![Javascript Profiling](https://assets.rspack.rs/rspack/assets/profiling-javascript.png)

#### Rust profiler

Rspack 的 Rust 代码通常执行在 tokio 线程里，选择 tokio 线程就可以查看 Rust 侧的耗时分布。

![Rust Profiling](https://assets.rspack.rs/rspack/assets/profiling-rust.png)

### Rsdoctor timeline

如果你需要分析 Loader 和 Plugin 耗时或者 Loader 的编译行为，可以利用 Rsdoctor 来查看：

![image](https://assets.rspack.rs/others/assets/rsdoctor/rsdoctor-loader-timeline.png)

参考 [Rsdoctor Compilation Analysis](/guide/optimization/profile#rsdoctor-%E7%9A%84%E7%BC%96%E8%AF%91%E5%88%86%E6%9E%90)

## Mac Xcode instruments

如果你使用的是 Mac，则 Xcode Instruments 工具可用于生成 CPU profile 文件。

![image](https://github.com/SyMind/rspack-dev-guide/assets/19852293/124e3aee-944a-4509-bb93-1c9213f026d3)

安装 Xcode Instruments，仅需要安装命令行工具：

```bash
xcode-select --install
```

对于普通 Rust 构建, [`cargo instruments`](https://github.com/cmyr/cargo-instruments) 可以用作胶水用于分析和创建 instruments 文件。

由于 Rspack 需要相当长的时间来构建，因此你可以使用以下过程而无需调用 `cargo Instruments`。
它具有相同的效果。

在根工作区的 `Cargo.toml`，在 `[profile.release]` 部分中打开调试符号并禁用符号剥离

```toml
[profile.release]
debug = 1 # debug info with line tables only
strip = false # do not strip symbols
```

然后构建项目

```bash
pnpm run build:cli:release
```

一旦项目构建完成，最后的二进制文件位于 `packages/rspack-cli/bin/rspack`。

`cargo Instruments` 在内部调用 `xcrun` 命令，
这意味着我们可以在我们自己使用 Rspack 的项目中运行以下命令。

```bash
xcrun xctrace record --template 'Time Profile' --output . --launch -- /path/to/rspack/packages/rspack-cli/bin/rspack build
```

它产生以下输出

```
Starting recording with the Time Profiler template. Launching process: rspack.
Ctrl-C to stop the recording
Target app exited, ending recording...
Recording completed. Saving output file...
Output file saved as: Launch_rspack_2023-04-24_11.32.06_9CFE3A63.trace
```

我们可以打开 trace file 通过

```bash
open Launch_rspack_2023-04-24_11.32.06_9CFE3A63.trace
```
