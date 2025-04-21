# Tracing

[`tracing`](https://crates.io/crates/tracing) is used to instrumenting Rspack.

The supported tracing levels for

- release builds are `INFO`, `WARN` and `ERROR`
- debug builds are `TRACE`, `DEBUG`, `INFO`, `WARN` and `ERROR`

Two ways to enable tracing:

- if you are using `@rspack/cli`, you can enable it by `RSPACK_PROFILE` environment variable.
- if you are using `@rspack/core` without `@rspack/cli`, you can enable it by `rspack.experiments.globalTrace.register` and `rspack.experiments.globalTrace.cleanup`, checkout [how we implement `RSPACK_PROFILE` in `@rspack/cli` with these two function](https://github.com/web-infra-dev/rspack/blob/9be47217b5179186b0825ca79990ab2808aa1a0f/packages/rspack-cli/src/utils/profile.ts#L219-L224) for more details.

### Chrome

[`tracing-chrome`](https://crates.io/crates/tracing-chrome) is supported for viewing tracing information graphically.

![image](https://github.com/SyMind/rspack-dev-guide/assets/19852293/1af08ba1-a2e9-4e3e-99ab-87c1e62e067b)

Setting the environment variable `RSPACK_PROFILE=TRACE=layer=chrome` before running Rspack, for example

```bash
RSPACK_PROFILE=TRACE=layer=chrome rspack build
```

produces a trace file (`.rspack-profile-${timestamp}-${pid}/trace.json`) in the current working directory.

The JSON trace file can be viewed in either `chrome://tracing` or [ui.perfetto.dev](https://ui.perfetto.dev).

### Terminal

Granular tracing event values can be viewed inside the terminal via `RSPACK_PROFILE=TRACE=layer=logger`, for example

```bash
RSPACK_PROFILE=TRACE=layer=logger rspack build
```

will print the options passed to Rspack as well as each individual tracing event.
