<picture>
  <img alt="Rspack Banner" src="https://assets.rspack.rs/rspack/rspack-banner.png">
</picture>

# @rspack/test-tools

> Rspack's internal test tools, don't use it directly in your project, it may contain breaking change in minor & patch release right now.

Test tools for rspack.

## Wasm tests

We expect to reuse the tests for Rspack wasm target as many as possible and we have managed to do it partially. Currently wasm tests should be run with an environment variable `WASM=1` under the following limitations:

1. Node 20 is required, which is consistent with the Node version in StackBlitz.
2. Set `maxWorkers` to `1`, `maxConcurrency` to `1` and disable concurrent mode to avoid flaky failures.
3. `forceExit` is needed.

Also check all the skipped testcases with `!process.env.WASM` in `test.filter.js`s and the skipped testsuits in `rstest.config.ts`s. They are divided into two categories:

1. Skip due to lacks of api support, such as tests related to swc wasm plugins, pnp and profiling. We skip them to avoid obsolete snapshot errors.
2. Skip temporarily and should be investigate in the future. There could be something wrong with test harness and rspack wasm itself. Since it could be time-consuming to figure out all of them so in this stage we use the wasm test ci to avoid the regression rather than improve its stability.

## Documentation

See [https://rspack.rs](https://rspack.rs) for details.

## License

Rspack is [MIT licensed](https://github.com/web-infra-dev/rspack/blob/main/LICENSE).
