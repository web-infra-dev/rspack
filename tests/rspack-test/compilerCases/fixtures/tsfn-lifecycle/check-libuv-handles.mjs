import assert from "node:assert/strict";
import path from "node:path";
import { MessageChannel } from "node:worker_threads";
import { rspack } from "@rspack/core";
import { createFsFromVolume, Volume } from "memfs";
import { closeCompiler, runCompiler } from "./helpers.mjs";
import {
  diagnostics,
  exceedsBaseline,
  settle,
  snapshot,
} from "./libuv-handles.mjs";

// Run by tsfn-lifecycle.js in an isolated Node process with --expose-gc so
// test-runner handles do not affect process.report.getReport().libuv counts.
// Guard against TSFN/handle leaks across compiler lifecycles and rebuilds, and
// against allocating an async handle for every module's loader data (#15636).
// Unreferenced handles still contribute libuv bookkeeping work even though they
// allow Node to exit, so successful process exit alone cannot detect these leaks.
// These are resource-count assertions, not event-loop latency benchmarks.

function createCompiler(count, fail = false, emit = true) {
  const state = { marker: "loader-hook", produced: 0, consumed: 0 };
  const loader = path.join(import.meta.dirname, "handle-loader.js");
  const compiler = rspack({
    context: import.meta.dirname,
    mode: "development",
    cache: false,
    devtool: false,
    // Distinct queries create distinct modules without generating fixture files.
    entry: Array.from({ length: count }, (_, i) => `./entry.js?module=${i}`),
    output: {
      path: path.join(import.meta.dirname, "output"),
      filename: "bundle.js",
    },
    module: {
      rules: [
        {
          test: /entry\.js$/,
          // Execution runs right to left. The builtin loader forces the JS
          // producer's additionalData to cross Rust before the JS consumer.
          use: [
            { loader, options: { role: "consumer" } },
            { loader: "builtin:test-passthrough-loader" },
            { loader, options: { role: "producer", fail } },
          ],
        },
      ],
    },
    plugins: [
      {
        apply(compiler) {
          // Skipping emit also skips the loader plugin's usual runner cleanup;
          // its close hook must release the runner even on this path.
          if (!emit) {
            compiler.hooks.shouldEmit.tap("LibuvHandles", () => false);
          }
          compiler.hooks.compilation.tap("LibuvHandles", (compilation) => {
            compiler.webpack.NormalModule.getCompilationHooks(
              compilation,
            ).loader.tap("LibuvHandles", (context) => {
              // Exercise loader-context state crossing the JS/Rust boundary,
              // including a closure that retains the loader context itself.
              context.handleState = state;
              context.handleContext = () => context;
            });
          });
        },
      },
    ],
  });
  compiler.outputFileSystem = createFsFromVolume(new Volume());
  return { compiler, state };
}

async function build(compiler, state, count, fail) {
  const produced = state.produced;
  const consumed = state.consumed;
  const stats = await runCompiler(compiler);
  assert.equal(
    stats.hasErrors(),
    fail,
    stats.toString({ all: false, errors: true }),
  );
  if (fail) {
    assert.match(
      stats.toString({ all: false, errors: true }),
      /intentional handle test failure/,
    );
  }
  // A cached/skipped loader could make handle counts look stable without
  // exercising the reference-management path on subsequent builds.
  assert.equal(
    state.produced - produced,
    count,
    "producer must execute for every module/build",
  );
  assert.equal(state.consumed - consumed, fail ? 0 : count);
}

async function lifecycle(count, fail = false, emit = true) {
  // Keep compiler/stats references inside this scope so the caller can GC them
  // after close, rather than counting resources retained by the test itself.
  const { compiler, state } = createCompiler(count, fail, emit);
  try {
    await build(compiler, state, count, fail);
  } finally {
    await closeCompiler(compiler);
  }
}

async function repeatedBuilds() {
  const { compiler, state } = createCompiler(4);
  try {
    // The first build clears hook caches on completion; rebuilds retain them
    // until the next compilation. Warm the rebuild path before comparing it.
    await build(compiler, state, 4, false);
    await build(compiler, state, 4, false);
    const liveBaseline = await settle("rebuild warmup");
    for (let round = 1; round <= 5; round++) {
      await build(compiler, state, 4, false);
      await settle(`rebuild ${round}`, liveBaseline, true);
    }
  } finally {
    await closeCompiler(compiler);
  }
}

async function scale(count, baseline) {
  const { compiler, state } = createCompiler(count);
  try {
    await build(compiler, state, count, false);
    // Measure while the compiler is still alive: per-module handles can hurt
    // the event loop during use even if they are all released after close.
    const current = await settle(`live compiler with ${count} modules`);
    return current.counts.async - baseline.counts.async;
  } finally {
    await closeCompiler(compiler);
  }
}

async function main() {
  const mode = process.argv[2];
  const fail = mode === "failure";
  const emit = mode !== "no-emit";
  // Initialize process-wide resources (such as the shared reference-deletion
  // TSFN) on the same success/error path before measuring compiler-owned growth.
  // settle() runs GC and drains async cleanup, requiring three matching samples.
  await lifecycle(4, fail, emit);
  const baseline = await settle("warmup");

  if (mode === "lifecycle" || fail || !emit) {
    // Every closed compiler must return to the warmed baseline, including when
    // a loader throws or emit is skipped. An elevated stable count still fails.
    for (let round = 1; round <= 5; round++) {
      await lifecycle(4, fail, emit);
      await settle(`${mode} ${round}: closed and collected`, baseline);
    }
  } else if (mode === "rebuild") {
    // Check both accumulation while reusing a compiler and cleanup after close.
    await repeatedBuilds();
    await settle("rebuild: closed and collected", baseline);
  } else if (mode === "scale") {
    // The parent compares the deltas from separate 4-module and 64-module
    // processes; use relative counts rather than platform-specific totals.
    const asyncDelta = await scale(Number(process.argv[3]), baseline);
    await settle("scale: closed and collected", baseline);
    console.log(JSON.stringify({ asyncDelta }));
  } else if (mode === "detector") {
    // Negative control: deliberately retain unref'ed async handles. Verify the
    // detector rejects them, then verify closing them restores the baseline.
    const { port1, port2 } = new MessageChannel();
    try {
      port1.unref();
      port2.unref();
      const current = snapshot();
      assert(
        exceedsBaseline(current, baseline, true),
        diagnostics("unref ports", baseline, current),
      );
      const addresses = new Set(baseline.handles.map((h) => h.address));
      const added = current.handles.filter(
        (h) => h.type === "async" && !addresses.has(h.address),
      );
      assert(added.length >= 2 && added.every((h) => !h.is_referenced));
      await assert.rejects(
        settle("intentional unref handle leak", baseline),
        /intentional unref handle leak/,
      );
    } finally {
      port1.close();
      port2.close();
    }
    await settle("closed ports", baseline);
  } else {
    throw new Error(`Unknown handle test mode: ${mode}`);
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
