# Module binding benchmarks

This harness measures Module snapshots, live accessors, native methods, and full
builds with no Module API access, identifier access, all fields, or JavaScript
loaders. Each build compiles 2,001 modules from an in-memory file system with a
fresh compiler and no build cache. API measurements run inside an awaited
`finishModules` tap, so native access stays within the hook lifetime.

Build the native binding with `pnpm run build:binding:ci` and JavaScript with
`pnpm run build:js`. Keep both artifacts from the same revision. Save each
binding binary and copy each core's `dist/` into a directory beside symlinks to
that core's `compiled/` and `node_modules/`. The copied core also needs its
`package.json` to retain ESM semantics.

Install the isolated benchmark dependencies and copy the runner:

```sh
npm install --prefix .bench/js-module-bindings --save-exact tinybench@5.1.0 @codspeed/tinybench-plugin@5.7.1
cp scripts/bench/module-bindings.cjs .bench/js-module-bindings/bench.cjs
git init --quiet .bench/js-module-bindings
```

The nested repository lets the CodSpeed Node plugin identify benchmark URIs
when the parent checkout is a Git worktree. It contains only ignored local
benchmark artifacts.

Run one revision at a time, with no builds or tests running in the background:

```sh
NAPI_RS_NATIVE_LIBRARY_PATH=/absolute/path/to/binding.node \
MODULE_BENCH_CORE=/absolute/path/to/dist/index.js \
RAYON_NUM_THREADS=4 \
codspeed run -r web-infra-dev/rspack -m walltime -- node .bench/js-module-bindings/bench.cjs
```

Use identical Node versions, build profiles, dependency versions, and thread
counts. Compare in A/B/B/A order and inspect both full-build and API results.
`MODULE_BENCH_COUNT` changes the number of leaf modules, and
`MODULE_BENCH_FILTER` selects benchmark names (for example, `api`, `development`,
or `js-loader`). The default is 2,000 leaf modules. Microbenchmarks repeat each
access 10,000 times; their reported time is for the entire batch.

The fixture covers Node-API overhead and loader creation before graph insertion.
It does not establish performance on other architectures, WASI, persistent-cache
workloads, or arbitrary applications.
