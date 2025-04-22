# Tracing

[`tracing`](https://crates.io/crates/tracing) is used to record the internal processes of Rspack compilation, which can be used for performance analysis as well as narrow down the location of a bug.

## Enabling Tracing

Tracing can be enabled in two ways:

- If using `@rspack/cli` or Rsbuild: Enable it by setting the `RSPACK_PROFILE` environment variable
- If directly using `@rspack/core`: Enable it through `rspack.experiments.globalTrace.register` and `rspack.experiments.globalTrace.cleanup`. You can check [how we implement `RSPACK_PROFILE` in `@rspack/cli`](https://github.com/web-infra-dev/rspack/blob/9be47217b5179186b0825ca79990ab2808aa1a0f/packages/rspack-cli/src/utils/profile.ts#L219-L224) for more information.

The generated `trace.json` file can be viewed and analyzed in [ui.perfetto.dev](https://ui.perfetto.dev/).

## Tracing Layer

Rspack supports two types of layers: `chrome` and `logger`:

- `chrome`: The default value, generates a trace.json file conforming to the [`chrome trace event`](https://docs.google.com/document/d/1CvAClvFfyA5R-PhYUmn5OOQtYMH4h6I0nSsKchNAySU/preview?tab=t.0#heading=h.yr4qxyxotyw) format, which can be exported to perfetto for complex performance analysis
- `logger`: Outputs logs directly to the terminal, suitable for simple log analysis or viewing compilation processes in CI environments

You can specify the layer through the `RSPACK_TRACE_LAYER` environment variable:

```sh
RSPACK_TRACE_LAYER=logger
# or
RSPACK_TRACE_LAYER=chrome
```

## Tracing Output

You can specify the output location for traces:

- The default output for the `logger` layer is `stdout`
- The default output for the `chrome` layer is `trace.json`

You can customize the output location through the `RSPACK_TRACE_OUTPUT` environment variable:

```sh
RSPACK_TRACE_LAYER=logger RSPACK_TRACE_OUTPUT=./log.txt rspack dev
RSPACK_TRACE_LAYER=chrome RSPACK_TRACE_OUTPUT=./perfetto.json rspack dev
```

## Tracing Filter

You can configure the data to be filtered through `RSPACK_PROFILE`. Rspack provides two preset options:

- `RSPACK_PROFILE=OVERVIEW`: The default value, only shows the core build process, generating a smaller JSON file
- `RSPACK_PROFILE=ALL`: Includes all trace events, used for more complex analysis, generating a larger JSON file

Apart from the presets, other strings will be passed directly to [Env Filter](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#example-syntax), supporting more complex filtering strategies:

### Tracing Level Filter

The supported tracing levels are: `TRACE`, `DEBUG`, `INFO`, `WARN`, and `ERROR`. You can filter by level:

```sh
# trace level is the highest level, outputting all logs
RSPACK_PROFILE=trace
# only output logs less than or equal to INFO level
RSPACK_PROFILE=info
```

### Module Level Filtering

```sh
# View rspack_resolver logs and output to terminal
RSPACK_TRACE_LAYER=logger RSPACK_PROFILE=rspack_resolver
```

### Mixed Filtering

EnvFilter supports mixed use of multiple filtering conditions to implement more complex filtering strategies:

```sh
# View WARN level logs in the rspack_core crate
RSPACK_PROFILE=rspack_core=warn
# Keep INFO level logs for other crates but turn off logs for rspack_resolver
RSPACK_PROFILE=info,rspack_core=off
```
