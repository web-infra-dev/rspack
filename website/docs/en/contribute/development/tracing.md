---
description: 'tracing is used to record the internal processes of Rspack compilation, which can be used for performance analysis as well as narrow down the location of a bug'
---

# Tracing

[`tracing`](https://crates.io/crates/tracing) is used to record the internal processes of Rspack compilation, which can be used for performance analysis as well as narrow down the location of a bug.

## Enabling tracing

Tracing can be enabled in two ways:

- If using [@rspack/cli](/api/cli) or Rsbuild: Enable it by setting the `RSPACK_PROFILE` environment variable:

```sh
# Rspack CLI
RSPACK_PROFILE=OVERVIEW rspack build # recommend
RSPACK_PROFILE=ALL rspack build # not recommend, may generate a large trace file for large projects

# Rsbuild
RSPACK_PROFILE=OVERVIEW rsbuild build
RSPACK_PROFILE=ALL rsbuild build
```

- If directly using `@rspack/core`: Enable it through `rspack.experiments.globalTrace.register` and `rspack.experiments.globalTrace.cleanup`. You can check how we implement [`RSPACK_PROFILE` in `@rspack/cli`](https://github.com/web-infra-dev/rspack/blob/main/packages/rspack-cli/src/utils/profile.ts) for more information.

When the `perfetto` layer is enabled, the generated `rspack.pftrace` file can be viewed and analyzed in [ui.perfetto.dev](https://ui.perfetto.dev/):

<img
  src="https://assets.rspack.rs/rspack/assets/rspack-v1-4-tracing.png"
  alt="tracing"
/>

## Tracing layer

Rspack supports two types of layers: `perfetto` and `logger`:

- `logger`: The default value, writes JSON Lines logs to a file, suitable for simple log analysis. In CI environments, upload or print the generated `rspack.log`, or set `RSPACK_TRACE_OUTPUT=stdout` / `stderr` explicitly to stream logger output.
- `perfetto`: Only available when using `@rspack-debug/core`. It generates a `rspack.pftrace` file conforming to the [`perfetto proto`](https://perfetto.dev/docs/reference/synthetic-track-event) format, which can be imported into Perfetto for complex performance analysis

`@rspack-debug/core` is a diagnostic variant of `@rspack/core` that includes extra debugging and tracing capabilities such as the `perfetto` layer. Use it when you need to collect Perfetto traces for local investigation, not as the default package for normal builds.

You can specify the layer through the `RSPACK_TRACE_LAYER` environment variable:

```sh
RSPACK_TRACE_LAYER=logger

# Only available with @rspack-debug/core
RSPACK_TRACE_LAYER=perfetto
```

## Tracing output

You can specify the output location for traces:

- The default output for the `logger` layer is `.rspack-profile-${timestamp}-${pid}/rspack.log`
- The default output for the `perfetto` layer is `.rspack-profile-${timestamp}-${pid}/rspack.pftrace`

You can customize the output location through the `RSPACK_TRACE_OUTPUT` environment variable:

```sh
RSPACK_TRACE_LAYER=logger RSPACK_TRACE_OUTPUT=log.txt rspack dev

# Only available with @rspack-debug/core
RSPACK_TRACE_LAYER=perfetto RSPACK_TRACE_OUTPUT=perfetto.pftrace rspack dev
```

When `RSPACK_TRACE_OUTPUT` is a relative file path, it is resolved inside the generated `.rspack-profile-${timestamp}-${pid}` directory. Absolute paths are used as-is. For the `logger` layer, set it to `stdout` or `stderr` explicitly if you need terminal output. The `perfetto` layer always requires a file path.

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
# View rspack_resolver logs
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
