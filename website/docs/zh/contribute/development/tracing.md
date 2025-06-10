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

- 如果直接使用 `@rspack/core`：可通过 `rspack.experiments.globalTrace.register` 和 `rspack.experiments.globalTrace.cleanup` 开启。你可以查看我们如何在 [`@rspack/cli` 中实现 `RSPACK_PROFILE`](https://github.com/web-infra-dev/rspack/blob/9be47217b5179186b0825ca79990ab2808aa1a0f/packages/rspack-cli/src/utils/profile.ts#L219-L224)获取更多信息。

生成的 `rspack.pftrace` 文件可以在 [ui.perfetto.dev](https://ui.perfetto.dev/) 中查看和分析。

## Tracing Layer

Rspack 支持 `perfetto` 和 `logger` 两种 layer：

- `perfetto`：默认值，生成符合 [`perfetto proto`](https://perfetto.dev/docs/reference/synthetic-track-event) 格式的 rspack.pftrace 文件，可导出到 perfetto 进行复杂的性能分析
- `logger`：直接在终端输出日志，适用于简单的日志分析或在 CI 环境中查看编译流程

可以通过 `RSPACK_TRACE_LAYER` 环境变量指定 layer：

```sh
RSPACK_TRACE_LAYER=logger
# 或
RSPACK_TRACE_LAYER=perfetto
```

## Tracing Output

可以指定 trace 的输出位置：

- `logger` layer 的默认输出为 `stdout`
- `perfetto` layer 的默认输出为 `rspack.pftrace`

通过 `RSPACK_TRACE_OUTPUT` 环境变量可以自定义输出位置：

```sh
RSPACK_TRACE_LAYER=logger RSPACK_TRACE_OUTPUT=log.txt rspack dev
RSPACK_TRACE_LAYER=perfetto RSPACK_TRACE_OUTPUT=perfetto.json rspack dev
```

## Tracing Filter

通过 `RSPACK_PROFILE` 可以配置需要过滤的数据。Rspack 提供了两个预设的 `preset`：

- `RSPACK_PROFILE=OVERVIEW`：默认值，只展示核心的构建流程，生成的 JSON 文件较小
- `RSPACK_PROFILE=ALL`：包含所有的 trace event，用于较为复杂的分析，生成的 JSON 文件较大

除了预设外，其他字符串都会透传给 [Env Filter](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#example-syntax)，支持更复杂的过滤策略：

### Tracing Level Filter

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
