## Tracing

[`tracing`](https://crates.io/crates/tracing) 被对 Rspack的编译的内部流程进行记录,其既可以用于性能分析也可以用于定位Bug。

### 开启 Tracing

两种方式开启 tracing:

- 如果你正在使用 `@rspack/cli`，你可以通过 `RSPACK_PROFILE` 环境变量来开启它。
- 如果你正在使用 `@rspack/core` 而不是 `@rspack/cli`，你可以通过 `rspack.experiments.globalTrace.register` 和 `rspack.experiments.globalTrace.cleanup` 开启，查看 [我们如何使用这两个函数在 `@rspack/cli` 中实现 `RSPACK_PROFILE`](https://github.com/web-infra-dev/rspack/blob/9be47217b5179186b0825ca79990ab2808aa1a0f/packages/rspack-cli/src/utils/profile.ts#L219-L224) 获取更多信息。
  生成的 `trace.jon` 可以在 [ui.perfetto.dev](https://ui.perfetto.dev/) 进行查看。

### Tracing Layer

Rspack 支持 `chrome` 和 `logger` 两种 layer,其作用区别为

- `logger`: 用于直接在终端输出日志，可以用于简单的日志分析，也可以用于在 CI 环境查看编译流程。
- `chrome`: 默认值，可以用于复杂的性能分析，生成符合 [`chrome trace event`](https://docs.google.com/document/d/1CvAClvFfyA5R-PhYUmn5OOQtYMH4h6I0nSsKchNAySU/preview?tab=t.0#heading=h.yr4qxyxotyw) 格式的trace.json,可以导出到 perfetto 进行分析。
  可以通过 `RSPACK_TRACE_LAYER` 来指定 layer。

```sh
RSPACK_TRACE_LAYER=logger
RSPACK_TRACE_LAYER=chrome
```

### Tracing Output

指定 trace 的文件输出到指定的文件, `logger` layer 的默认值为 `stdout`, `chrome` layer 的默认值为 `trace.json`。可以通过 `RSPACK_TRACE_OUTPUT` 来指定输出文件,可以配合 `RSPACK_TRACE_LAYER` 一起使用。

```sh
RSPACK_TRACE_LAYER=logger RSPACK_TRACE_OUTPUT=log.txt rspack dev
RSPACK_TRACE_LAYER=chrome RSPACK_TRACE_OUTPUT=perfetto.json rspack dev
```

### Tracing Filter

可以通过 `RSPACK_PROFILE` 来配置需要过滤的数据，`Rspack` 默认提供了两个预置的 `preset`

- `RSPACK_PROFILE=OVERVIEW`: 默认值，只展示核心的构建流程，生成的 json 文件较小
- `RSPACK_PROFIE=ALL`: 包含所有的 trace event，用于较为复杂的分析，生成的 json 文件较大
  除了预设 `preset` 之外的其他字符串都会透传给 [Env Filter](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#example-syntax) 支持更复杂的过滤策略。一些常用的 Filter 如下

#### Tracing Level Filter

支持 tracing 的等级有： `TRACE`, `DEBUG`, `INFO`, `WARN` and `ERROR`, envFilter 支持通过 level 进行过滤。

```sh
RSPACK_PROFIE=trace  # trace level 是最大的level，输出所有日志
RSPACK_PROFIE=info # 只输出小于等于 INFO level的日志
```

#### Module Level filter

```sh
RSPACK_TRACE_LAYER=logger RSPACK_PROFILE=rspack_resolver # 查看 rspack_resolver 的日志，并输出到终端
```

#### Mixed Module Level Filter & Tracing Level Filter

EnvFilter 还可以将上面几种 filter 进行混用，支持更复杂的过滤

```sh

 RSPACK_PROFIE=rspack_core=warn # 查看 rspack_core crate 里的 WARN level 的日志
 RSPACK_PROFILE=info,rspack_core=off # 保留其他 crate 的INFO level 日志但是关闭 rspack_resolver 的日志
```
