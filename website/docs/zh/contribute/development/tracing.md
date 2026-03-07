# Tracing

[`tracing`](https://crates.io/crates/tracing) 用于记录 Rspack 内部的编译流程，既可用于性能分析，也可用于定位 Bug。

## 开启 Tracing

可以通过以下两种方式开启 tracing：

- 如果使用 [@rspack/cli](/api/cli) 或 Rsbuild：通过设置 `RSPACK_PROFILE` 环境变量来开启：

```sh
# Rspack CLI
RSPACK_PROFILE=OVERVIEW rspack build # 推荐
RSPACK_PROFILE=ALL rspack build # 不推荐，大项目的 rspack.pftrace 体积可能非常大

# Rsbuild
RSPACK_PROFILE=OVERVIEW rsbuild build
RSPACK_PROFILE=ALL rsbuild build
```

- 如果直接使用 `@rspack/core`：可通过 `rspack.experiments.globalTrace.register` 和 `rspack.experiments.globalTrace.cleanup` 开启。你可以查看我们如何在 [`@rspack/cli` 中实现 `RSPACK_PROFILE`](https://github.com/web-infra-dev/rspack/blob/main/packages/rspack-cli/src/utils/profile.ts) 获取更多信息。

使用默认的 `perfetto` layer 时，生成的 `rspack.pftrace` 文件可以在 [ui.perfetto.dev](https://ui.perfetto.dev/) 中查看和分析：

<img
  src="https://assets.rspack.rs/rspack/assets/rspack-v1-4-tracing.png"
  alt="tracing"
/>

## Tracing layer

Rspack 支持 `perfetto`、`logger` 和 `hotpath` 三种 layer：

- `perfetto`：默认值，生成符合 [`perfetto proto`](https://perfetto.dev/docs/reference/synthetic-track-event) 格式的 rspack.pftrace 文件，可导出到 perfetto 进行复杂的性能分析
- `logger`：直接在终端输出结构化日志，适用于简单的日志分析或在 CI 环境中查看编译流程
- `hotpath`：按名称聚合 tracing span，并输出带有 `Calls`、`Avg`、`P95`、`Total` 和 `% Total` 列的 hotpath 风格表格，适合在终端中快速查看热点。如果输出路径以 `.json` 结尾，则会改为输出便于 diff 的 JSON 报告

可以通过 `RSPACK_TRACE_LAYER` 环境变量指定 layer：

```sh
RSPACK_TRACE_LAYER=logger
# 或
RSPACK_TRACE_LAYER=hotpath
# 或
RSPACK_TRACE_LAYER=perfetto
```

## Tracing output

可以指定 trace 的输出位置：

- `logger` 和 `hotpath` layer 的默认输出为 `stdout`
- `perfetto` layer 的默认输出为 `.rspack-profile-${timestamp}-${pid}` 目录下的 `rspack.pftrace`
- 当 `RSPACK_TRACE_OUTPUT` 为相对路径时，`@rspack/cli` 会将它解析到生成的 `.rspack-profile-${timestamp}-${pid}` 目录下
- 对于 `hotpath` layer，如果输出路径以 `.json` 结尾，则会输出带有 `avg_raw`、`total_raw` 和 `percent_total_raw` 等原始数值字段的格式化 JSON 报告

通过 `RSPACK_TRACE_OUTPUT` 环境变量可以自定义输出位置：

```sh
RSPACK_TRACE_LAYER=logger RSPACK_TRACE_OUTPUT=log.txt rspack dev
RSPACK_TRACE_LAYER=hotpath RSPACK_TRACE_OUTPUT=hotpath.txt rspack dev
RSPACK_TRACE_LAYER=hotpath RSPACK_TRACE_OUTPUT=hotpath.json rspack dev
RSPACK_TRACE_LAYER=perfetto RSPACK_TRACE_OUTPUT=rspack.pftrace rspack dev
```

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
# 查看 rspack_resolver 的日志，并输出到终端
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
