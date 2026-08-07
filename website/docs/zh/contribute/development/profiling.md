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
pnpm build:binding:profiling
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

## 使用 jemalloc 进行内存分析

在支持的原生平台上，`@rspack-debug/core` 默认使用带 heap profiling 能力的 jemalloc。只有通过 `_RJEM_MALLOC_CONF` 启用采样后才会输出 profile，因此 debug package 在未配置该变量时仍可正常使用，不会产生 profile 文件。

:::note
支持的 Linux 目标和 macOS 上的 debug package 支持 jemalloc profiling。Windows MSVC、s390x Linux 和 Wasm package 不支持该能力，会继续使用默认 allocator。常规的 `@rspack/core` package 不受影响。
:::

### 安装 debug package

在需要分析的项目中，通过 package manager override 将 `@rspack/core` 替换为 `@rspack-debug/core`。以 pnpm 为例：

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

添加 override 后重新安装项目依赖：

```sh
pnpm install
```

如果正在排查特定版本的问题，请将 `latest` 替换为项目原本使用的 `@rspack/core` 版本。分析结束后应移除 override。

### 采集 heap profile

先创建输出目录，再通过 jemalloc profiling 运行项目原本的构建命令。对于直接使用 Rspack CLI 的项目：

```sh
mkdir -p /tmp/rspack-jemalloc

cd /path/to/project
_RJEM_MALLOC_CONF='prof:true,prof_active:true,prof_final:true,lg_prof_sample:19,lg_prof_interval:26,prof_prefix:/tmp/rspack-jemalloc/rspack' \
pnpm exec rspack build
```

如果项目使用 Rsbuild 或其他基于 Rspack 的工具，请保留同一个环境变量，并运行该工具原本的构建命令。

`tikv-jemallocator` 会为运行时配置变量添加前缀，因此需要使用 `_RJEM_MALLOC_CONF`，而不是 `MALLOC_CONF`。

上述配置的含义如下：

- `prof:true` 启用 heap profiling，必须在进程启动时设置。
- `prof_active:true` 在进程启动后立即开始采样。
- `prof_final:true` 在进程退出时输出最终的 `.f.heap` 文件。
- `lg_prof_sample:19` 表示大约每 512 KiB 分配采样一次。减小该值可以获得更多细节，但会增加开销。
- `lg_prof_interval:26` 表示大约每 64 MiB allocation activity 输出一个 `.i.heap` 文件。
- `prof_prefix` 控制 profile 文件的输出位置。

需要分析累计分配量时，可以添加 `prof_accum:true`。它会增加 profiler 自身的内存开销，因此只分析 live allocations 时应当省略。通常不要使用 `prof_gdump:true`，因为 high-water mark 的小幅增长就可能触发大量 dump，明显干扰测量结果。

### 生成函数级内存分配图

安装 jemalloc 5.x 提供的 `jeprof` 和 Graphviz。先找到 `@rspack-debug/core` 实际加载的 native binding，再将该 `.node` 文件作为包含 Rust 符号的 program 参数传给 `jeprof`：

```sh
RSPACK_BINDING=$(node -e 'require("@rspack/core"); const binding = Object.keys(require.cache).find(file => /rspack\..+\.node$/.test(file)); if (!binding) throw new Error("Rspack native binding not found"); process.stdout.write(binding)')

jeprof --show_bytes --functions --exclude='_+rjem_' --svg \
  "$RSPACK_BINDING" \
  /tmp/rspack-jemalloc/rspack.<pid>.<sequence>.heap \
  > rspack-memory.svg
```

生成按累计调用路径排序的文本报告：

```sh
jeprof --show_bytes --functions --exclude='_+rjem_' --text --cum \
  "$RSPACK_BINDING" \
  /tmp/rspack-jemalloc/rspack.<pid>.<sequence>.heap
```

如果采集时开启了 `prof_accum:true`，可以通过 `--alloc_space` 查看 build 期间发生的所有采样分配：

```sh
jeprof --alloc_space --show_bytes --functions --exclude='_+rjem_' --svg \
  "$RSPACK_BINDING" \
  /tmp/rspack-jemalloc/rspack.<pid>.<sequence>.heap \
  > rspack-alloc-space.svg
```

默认的 `inuse_space` 表示 dump 时仍然存活的分配；`alloc_space` 表示累计 allocation traffic，适合查找临时分配抖动，但它不是内存峰值。jemalloc 只覆盖 Rspack binding 的 Rust global allocator；Node.js、V8、原生库、内存映射和 profiler 元数据同样会计入进程 RSS。如果还需要整个进程的峰值 RSS，可以在 macOS 上使用 `/usr/bin/time -l`，在 Linux 上使用 `/usr/bin/time -v`。

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

参考 [Rsdoctor Compilation Analysis](/guide/optimization/profile#使用-rsdoctor-分析)

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
