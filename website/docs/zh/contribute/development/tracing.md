---
description: '介绍 Rspack 中 Tracing 的使用方式'
---

# Tracing

[`tracing`](https://crates.io/crates/tracing) 用于记录 Rspack 内部的编译流程，既可用于性能分析，也可用于定位 Bug。

## 开启 Tracing

可以通过以下两种方式开启 tracing：

- 如果使用 [@rspack/cli](/api/cli) 或 Rsbuild：通过设置 `RSPACK_PROFILE` 环境变量来开启：

```sh
# Rspack CLI
RSPACK_PROFILE=OVERVIEW rspack build # 推荐
RSPACK_PROFILE=ALL rspack build # 不推荐，大项目可能会生成较大的 trace 文件

# Rsbuild
RSPACK_PROFILE=OVERVIEW rsbuild build
RSPACK_PROFILE=ALL rsbuild build
```

- 如果直接使用 `@rspack/core`：可通过 `rspack.experiments.globalTrace.register` 和 `rspack.experiments.globalTrace.cleanup` 开启。你可以查看我们如何在 [`@rspack/cli` 中实现 `RSPACK_PROFILE`](https://github.com/web-infra-dev/rspack/blob/main/packages/rspack-cli/src/utils/profile.ts) 获取更多信息。

启用 `perfetto` layer 后，生成的 `rspack.pftrace` 文件可以在 [ui.perfetto.dev](https://ui.perfetto.dev/) 中查看和分析：

<img
  src="https://assets.rspack.rs/rspack/assets/rspack-v1-4-tracing.png"
  alt="tracing"
/>

## Tracing layer

Rspack 支持 `perfetto` 和 `logger` 两种 layer：

- `logger`：默认值，将 JSON Lines 格式的日志写入文件，适用于简单的日志分析。在 CI 环境中，可以上传或打印生成的 `rspack.log`，也可以显式设置 `RSPACK_TRACE_OUTPUT=stdout` / `stderr` 将 logger 输出流式写到终端。
- `perfetto`：仅在使用 `@rspack-debug/core` 时可用，生成符合 [`perfetto proto`](https://perfetto.dev/docs/reference/synthetic-track-event) 格式的 `rspack.pftrace` 文件，可导入到 Perfetto 进行复杂的性能分析

`@rspack-debug/core` 是 `@rspack/core` 的诊断版本，包含额外的调试和 tracing 能力，例如 `perfetto` layer。当你需要为本地问题排查收集 Perfetto trace 时使用它，不建议把它作为常规构建的默认依赖。

可以通过 `RSPACK_TRACE_LAYER` 环境变量指定 layer：

```sh
RSPACK_TRACE_LAYER=logger

# 仅适用于 @rspack-debug/core
RSPACK_TRACE_LAYER=perfetto
```

## Tracing output

可以指定 trace 的输出位置：

- `logger` layer 的默认输出为 `.rspack-profile-${timestamp}-${pid}/rspack.log`
- `perfetto` layer 的默认输出为 `.rspack-profile-${timestamp}-${pid}/rspack.pftrace`

通过 `RSPACK_TRACE_OUTPUT` 环境变量可以自定义输出位置：

```sh
RSPACK_TRACE_LAYER=logger RSPACK_TRACE_OUTPUT=log.txt rspack dev

# 仅适用于 @rspack-debug/core
RSPACK_TRACE_LAYER=perfetto RSPACK_TRACE_OUTPUT=perfetto.pftrace rspack dev
```

当 `RSPACK_TRACE_OUTPUT` 是相对文件路径时，它会解析到生成的 `.rspack-profile-${timestamp}-${pid}` 目录下。绝对路径会按原样使用。对于 `logger` layer，如果需要输出到终端，可以显式设置为 `stdout` 或 `stderr`。`perfetto` layer 始终需要文件路径。

## Tracing filter

通过 `RSPACK_PROFILE` 可以配置需要过滤的数据。Rspack 提供了两个预设的 `preset`：

- `RSPACK_PROFILE=OVERVIEW`：默认值，只展示核心的构建流程，生成的 JSON 文件较小
- `RSPACK_PROFILE=ALL`：包含所有的 trace event，用于较为复杂的分析，生成的 JSON 文件较大

除了预设外，其他字符串都会透传给 [Env Filter](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#example-syntax)，支持更复杂的过滤策略：

### Tracing level filter

支持的 tracing 等级有：`TRACE`、`DEBUG`、`INFO`、`WARN` 和 `ERROR`。可以通过等级进行过滤：

```sh
# trace level 是最高级别，输出所有日志
RSPACK_PROFILE=trace
# 只输出小于等于 INFO level 的日志
RSPACK_PROFILE=info
```

### 模块级别过滤

```sh
# 查看 rspack_resolver 的日志
RSPACK_TRACE_LAYER=logger RSPACK_PROFILE=rspack_resolver
```

### 混合过滤

EnvFilter 支持混合使用多种过滤条件，实现更复杂的过滤策略：

```sh
# 查看 rspack_core crate 里的 WARN level 的日志
RSPACK_PROFILE=rspack_core=warn
# 保留其他 crate 的 INFO level 日志但关闭 rspack_resolver 的日志
RSPACK_PROFILE=info,rspack_core=off
```
