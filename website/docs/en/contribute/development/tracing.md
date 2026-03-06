# Tracing

[`tracing`](https://crates.io/crates/tracing) is used to record the internal processes of Rspack compilation, which can be used for performance analysis as well as narrow down the location of a bug.

## Enabling tracing

Tracing can be enabled in two ways:

- If using [@rspack/cli](/api/cli) or Rsbuild: Enable it by setting the `RSPACK_PROFILE` environment variable:

```sh
# Rspack CLI
RSPACK_PROFILE=OVERVIEW rspack build # recommend
RSPACK_PROFILE=ALL rspack build # not recommend, may generate too large rspack.pftrace for large projects

# Rsbuild
RSPACK_PROFILE=OVERVIEW rsbuild build
RSPACK_PROFILE=ALL rsbuild build
```

- If directly using `@rspack/core`: Enable it through `rspack.experiments.globalTrace.register` and `rspack.experiments.globalTrace.cleanup`. You can check how we implement [`RSPACK_PROFILE` in `@rspack/cli`](https://github.com/web-infra-dev/rspack/blob/main/packages/rspack-cli/src/utils/profile.ts) for more information.

When using the default `perfetto` layer, the generated `rspack.pftrace` file can be viewed and analyzed in [ui.perfetto.dev](https://ui.perfetto.dev/):

<img
  src="https://assets.rspack.rs/rspack/assets/rspack-v1-4-tracing.png"
  alt="tracing"
/>

## Tracing layer

Rspack supports three types of layers: `perfetto`, `logger`, and `hotpath`:

- `perfetto`: The default value, generates a rspack.pftrace file conforming to the [`perfetto proto`](https://perfetto.dev/docs/reference/synthetic-track-event) format, which can be exported to perfetto for complex performance analysis
- `logger`: Outputs structured logs directly to the terminal, suitable for simple log analysis or viewing compilation processes in CI environments
- `hotpath`: Aggregates tracing spans by name and prints a hotpath-style table with `Calls`, `Avg`, `P95`, `Total`, and `% Total`, suitable for quick terminal inspection without opening Perfetto. If the output path ends with `.json`, it emits a diff-friendly JSON report instead

You can specify the layer through the `RSPACK_TRACE_LAYER` environment variable:

```sh
RSPACK_TRACE_LAYER=logger
# or
RSPACK_TRACE_LAYER=hotpath
# or
RSPACK_TRACE_LAYER=perfetto
```

## Tracing output

You can specify the output location for traces:

- The default output for the `logger` and `hotpath` layers is `stdout`
- The default output for the `perfetto` layer is `rspack.pftrace` inside `.rspack-profile-${timestamp}-${pid}`
- When `RSPACK_TRACE_OUTPUT` is a relative file path, `@rspack/cli` resolves it under the generated `.rspack-profile-${timestamp}-${pid}` directory
- For the `hotpath` layer, output paths ending with `.json` produce a pretty-printed JSON report with raw numeric fields such as `avg_raw`, `total_raw`, and `percent_total_raw`

You can customize the output location through the `RSPACK_TRACE_OUTPUT` environment variable:

```sh
RSPACK_TRACE_LAYER=logger RSPACK_TRACE_OUTPUT=./log.txt rspack dev
RSPACK_TRACE_LAYER=hotpath RSPACK_TRACE_OUTPUT=./hotpath.txt rspack dev
RSPACK_TRACE_LAYER=hotpath RSPACK_TRACE_OUTPUT=./hotpath.json rspack dev
RSPACK_TRACE_LAYER=perfetto RSPACK_TRACE_OUTPUT=./perfetto.pftrace rspack dev
```

## Tracing filter

You can configure the data to be filtered through `RSPACK_PROFILE`. Rspack provides two preset options:

- `RSPACK_PROFILE=OVERVIEW`: The default value, only shows the core build process, generating a smaller JSON file
- `RSPACK_PROFILE=ALL`: Includes all trace events, used for more complex analysis, generating a larger JSON file

Apart from the presets, other strings will be passed directly to [Env Filter](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#example-syntax), supporting more complex filtering strategies:

### Tracing level filter

The supported tracing levels are: `TRACE`, `DEBUG`, `INFO`, `WARN`, and `ERROR`. You can filter by level:

```sh
# trace level is the highest level, outputting all logs
RSPACK_PROFILE=trace
# only output logs less than or equal to INFO level
RSPACK_PROFILE=info
```

### Module level filtering

```sh
# View rspack_resolver logs and output to terminal
RSPACK_TRACE_LAYER=logger RSPACK_PROFILE=rspack_resolver
```

### Mixed filtering

EnvFilter supports mixed use of multiple filtering conditions to implement more complex filtering strategies:

```sh
# View WARN level logs in the rspack_core crate
RSPACK_PROFILE=rspack_core=warn
# Keep INFO level logs for other crates but turn off logs for rspack_resolver
RSPACK_PROFILE=info,rspack_core=off
```
