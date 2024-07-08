# Profiling

在本节中，我们将探讨如何分析 Rspack Profile 以识别性能瓶颈。

通过检查 Rspack 将时间花在哪里，我们可以深入了解如何提高性能。

由于不同的分析器有不同的优势，使用多个分析器是一个好的选择。

<!-- toc -->

## Tracing

[`tracing`](https://crates.io/crates/tracing) 被用于度量（instrumenting） Rspack。

被支持 tracing 等级有：

- release 版本是 `INFO`, `WARN` and `ERROR`
- debug 版本是 `TRACE`, `DEBUG`, `INFO`, `WARN` and `ERROR`

两种方式开启 tracing:

- 如果你正在使用 `@rspack/cli`，你可以通过 `RSPACK_PROFILE` 环境变量来开启它。
- 如果你正在使用 `@rspack/core` 而不是 `@rspack/cli`，你可以通过 `rspack.experiments.globalTrace.register` 和 `rspack.experiments.globalTrace.cleanup` 开启，查看 [我们如何使用这两个函数在 `@rspack/cli` 中实现 `RSPACK_PROFILE`](https://github.com/web-infra-dev/rspack/blob/9be47217b5179186b0825ca79990ab2808aa1a0f/packages/rspack-cli/src/utils/profile.ts#L219-L224) 获取更多信息。

### Chrome

[`tracing-chrome`](https://crates.io/crates/tracing-chrome) 支持以图形方式查看 tracing 信息。

![image](https://github.com/SyMind/rspack-dev-guide/assets/19852293/1af08ba1-a2e9-4e3e-99ab-87c1e62e067b)

在运行 Rspack 之前设置环境变量 `RSPACK_PROFILE=TRACE=layer=chrome`，例如

```bash
RSPACK_PROFILE=TRACE=layer=chrome rspack build
```

产生了一个 trace 文件 (`.rspack-profile-${timestamp}-${pid}/trace.json`) 在目前的工作目录。

JSON 跟踪文件可以在 `chrome://tracing` 或者 [ui.perfetto.dev](https://ui.perfetto.dev) 查看。

### Terminal

可以通过 `RSPACK_PROFILE=TRACE=layer=logger` 在终端内查看细粒度的 tracing 事件数值，例如

```bash
RSPACK_PROFILE=TRACE=layer=logger rspack build
```

将打印传递给 Rspack 的选项以及每个单独的 tracing 事件.

### Nodejs Profiling

如果我们发现性能瓶颈在JS端（比如js loader），那么我们需要进一步分析 js 端，可以使用 Nodejs Profiling 来分析。例如

```bash
node --cpu-prof {rspack_bin_path} -c rspack.config.js
```

或者

```bash
RSPACK_PROFILE=JSCPU rspack build
```

这将生成一个 cpu 配置文件，例如 `CPU.20230522.154658.14577.0.001.cpuprofile`，并且我们可以使用 speedscope 来可视化 profile，例如

```bash
npm install -g speedscope
speedscope CPU.20230522.154658.14577.0.001.cpuprofile
```

## Mac Xcode Instruments

如果您使用的是 Mac，则 Xcode Instruments 工具可用于生成 CPU profile 文件。

![image](https://github.com/SyMind/rspack-dev-guide/assets/19852293/124e3aee-944a-4509-bb93-1c9213f026d3)

安装 Xcode Instruments，仅需要安装命令行工具：

```bash
xcode-select --install
```

对于普通 Rust 构建, [`cargo instruments`](https://github.com/cmyr/cargo-instruments) 可以用作胶水用于分析和创建 tracing 文件。

由于 Rspack 需要相当长的时间来构建，因此您可以使用以下过程而无需调用 `cargo Instruments`。
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
