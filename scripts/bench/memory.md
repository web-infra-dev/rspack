# Compare peak memory of native builds

Use this harness to compare full-build peak RSS, including JavaScript and native allocations. Run it on Linux with matching JavaScript packages and native bindings.

1. Prepare the inputs and JavaScript packages:

   ```sh
   pnpm install --frozen-lockfile
   pnpm run build:js
   pnpm run bench:prepare
   mkdir -p .bench/memory
   ```

2. Before applying the Rust change, build and save the baseline:

   ```sh
   pnpm run build:binding:release
   cp crates/node_binding/rspack.linux-x64-gnu.node .bench/memory/baseline.node
   ```

3. Apply the change, then build and save the candidate:

   ```sh
   pnpm run build:binding:release
   cp crates/node_binding/rspack.linux-x64-gnu.node .bench/memory/candidate.node
   ```

4. Run five alternating builds per binding:

   ```sh
   node scripts/bench/memory.mjs \
     --baseline .bench/memory/baseline.node \
     --candidate .bench/memory/candidate.node \
     --runs 5 \
     --report .bench/memory/production-results.json
   ```

   Each build must complete without errors, compile the same number of modules, and emit byte-identical assets. The `threejs-10x` fixture also executes both vector checks in each of its ten Three.js copies.

5. Repeat with `--mode development` and with `--devtool false` to check adjacent configurations. Give each run a different `--report` path.

Keep both sides on the same machine and use the same inputs, thread counts, allocator settings, and build profile. The harness reverses the pair order on alternate iterations so the candidate does not always run second. Identical native binding digests are rejected. The harness defaults all three worker pools to 16 threads. It records overrides from `RAYON_NUM_THREADS`, `TOKIO_WORKER_THREADS`, and `RSPACK_BLOCKING_THREADS`.

Inspect the report's median, minimum, and maximum rather than comparing single runs. The input digest, native binding digests, module counts, asset hashes, errors, and runtime check counts identify the work performed.

Peak RSS is the fresh process's high-water mark at `compiler.run` completion, before stats materialization and output validation. It includes package loading and compiler initialization. Build wall time starts immediately before `compiler.run`; it excludes compiler shutdown and validation. No forced garbage collection or allocator purging runs.

For diagnosis only, use `--binding <path> --runs 1 --profile` to collect RSS at compilation hooks. Set `RSPACK_ALLOCATIVE_DIR` only with an allocative-enabled binding and `--profile`. Do not mix these runs into the release comparison.

The default workload is the repository's scaled Three.js fixture, with production optimization and full source maps. A result on it does not establish the same reduction for every application or configuration. The harness shares one output directory, so run only one instance at a time.
