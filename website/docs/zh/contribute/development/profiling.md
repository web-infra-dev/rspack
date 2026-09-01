---
description: '性能分析应基于包含调试信息的发布版本进行。这种方法既能确保性能结果的准确性，又能提供充足的调试信息用于分析。'
---

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

## 内存分析

内存分析器在 allocator 边界记录分配。Rspack 的常规 release 包使用 mimalloc，而 `@rspack-debug/core` 使用 system allocator。采集 Heaptrack 数据或动态注入 jemalloc 时，应使用 `@rspack-debug/core`。

在需要分析的项目中，按照[调试文档](/contribute/development/debugging#使用-rspack-debugcore)将 `@rspack/core` override 为相同版本的 `@rspack-debug/core`，然后重新安装依赖。例如，使用 pnpm 的项目依赖 `@rspack/core@2.1.0` 时：

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

| 分析器             | 输出格式 | 分析工具                           | 适用场景                 |
| ------------------ | -------- | ---------------------------------- | ------------------------ |
| Heaptrack          | `*.gz`   | `heaptrack_print`、`heaptrack_gui` | 分配调用栈和临时分配     |
| jemalloc profiling | `*.heap` | `jeprof`                           | 存活内存和累计分配火焰图 |

:::warning
Heaptrack 和 jemalloc 使用不同的数据格式，`jeprof` 无法读取 Heaptrack 数据。不要在同一次运行中同时启用 Heaptrack 和 jemalloc profiling。
:::

### Heaptrack

[Heaptrack](https://github.com/KDE/heaptrack) 通过拦截 system allocator 调用记录分配调用栈。使用 Linux 发行版的包管理器安装 Heaptrack，然后执行：

```sh
heaptrack --record-only -o ./rspack-heaptrack \
  node ./node_modules/@rspack/cli/bin/rspack.js build
```

可以在终端或 GUI 中分析生成的数据：

```sh
heaptrack_print ./rspack-heaptrack.gz | less
heaptrack_gui ./rspack-heaptrack.gz
```

Heaptrack 退出时会打印实际的输出文件名。`--record-only` 可以避免自动打开 GUI，适合 WSL、容器和远程终端。

`@rspack-debug/core` 使用 system allocator，因此 Heaptrack 能够采集 Rspack native binding 的内存分配。常规 release 包使用 mimalloc，会绕过 system allocator 的拦截点，因此其中的 Rust 分配数据不完整。根据 Heaptrack 版本不同，GUI 可能显示以 `_R` 开头的 Rust v0 mangled 符号，而不是 demangle 后的名称；这只影响展示，不影响已记录的调用栈。如需生成 demangle 后的文本报告，可以安装 [`rustfilt`](https://github.com/luser/rustfilt) 并执行：

```sh
heaptrack_print ./rspack-heaptrack.gz | rustfilt | less
```

### jemalloc profiling

在 Linux 上，可以通过 `LD_PRELOAD` 将使用 system allocator 的构建重定向到启用了 profiling 的 jemalloc 动态库。请使用发行版的软件包安装 jemalloc、`jeprof` 和 Graphviz。在 Debian 或 Ubuntu 中，`libjemalloc2` 提供动态库，开发包通常会提供 profiling 工具。

例如，在 Debian 或 Ubuntu 中执行：

```sh
sudo apt install heaptrack libjemalloc-dev graphviz
```

定位动态库并采集 profile：

```sh
mkdir -p /tmp/rspack-jemalloc
JEMALLOC=$(ldconfig -p | awk '/libjemalloc.so.2/{print $NF; exit}')

MALLOC_CONF='prof:true,prof_active:true,prof_final:true,lg_prof_sample:19,lg_prof_interval:26,prof_prefix:/tmp/rspack-jemalloc/rspack' \
  LD_PRELOAD="$JEMALLOC" \
  node ./node_modules/@rspack/cli/bin/rspack.js build
```

主要配置项如下：

- `prof:true`：启用 profiling。
- `prof_active:true`：进程启动后立即开始采样。
- `prof_final:true`：进程退出时写入最终 dump。
- `lg_prof_sample:19`：大约每分配 512 KiB 采样一次。
- `lg_prof_interval:26`：每产生大约 64 MiB 分配活动写入一次 dump。
- `prof_prefix`：指定 profile 文件的输出位置。

查找项目实际加载的 Rspack native binding：

```sh
RSPACK_BINDING=$(node -e 'require("@rspack/core"); const binding = Object.keys(require.cache).find(file => /rspack\..+\.node$/.test(file)); if (!binding) throw new Error("Rspack native binding not found"); process.stdout.write(binding)')
PROFILE=$(ls -t /tmp/rspack-jemalloc/rspack.*.heap | head -n 1)
```

选择一个 dump 并生成 SVG，然后使用浏览器打开：

```sh
jeprof --show_bytes --functions --svg \
  "$RSPACK_BINDING" \
  "$PROFILE" \
  > rspack-memory.svg
```

生成按累计内存排序的文本报告：

```sh
jeprof --show_bytes --functions --text --cum \
  "$RSPACK_BINDING" \
  "$PROFILE"
```

默认的 `inuse_space` 报告表示写入 dump 时仍然存活的内存。分析累计分配流量时，在 `MALLOC_CONF` 中增加 `prof_accum:true`，并向 `jeprof` 传入 `--alloc_space`。累计分配字节数并不等于峰值内存。

jemalloc 只统计经过 jemalloc 的分配。Node.js、V8、其他 native library、内存映射以及 profiler 自身的元数据也会占用进程 RSS。分析总内存或峰值内存时，应同时使用 `/usr/bin/time -v` 对照进程级数据。

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

参考 [Rsdoctor Compilation Analysis](/guide/diagnostics/profile#使用-rsdoctor-分析)

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
