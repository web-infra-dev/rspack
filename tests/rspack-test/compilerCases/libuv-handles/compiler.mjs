import assert from "node:assert/strict";
import path from "node:path";
import { rspack } from "@rspack/core";
import { createFsFromVolume, Volume } from "memfs";
import {
  runCompiler,
  closeCompiler,
} from "@rspack/test-tools/helper/lifecycle";
export function createCompiler(count, fail = false, emit = true) {
  const loaderState = {
    marker: "loader-hook",
    producerCalls: 0,
    consumerCalls: 0,
  };
  const loader = path.join(import.meta.dirname, "loader.js");
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
              context.loaderTestState = loaderState;
              context.getCapturedLoaderContext = () => context;
            });
          });
        },
      },
    ],
  });
  compiler.outputFileSystem = createFsFromVolume(new Volume());
  return { compiler, loaderState };
}

export async function buildAndCheckLoaders(compiler, loaderState, count, fail) {
  const producerCallsBeforeBuild = loaderState.producerCalls;
  const consumerCallsBeforeBuild = loaderState.consumerCalls;
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
    loaderState.producerCalls - producerCallsBeforeBuild,
    count,
    "producer must execute for every module/build",
  );
  assert.equal(
    loaderState.consumerCalls - consumerCallsBeforeBuild,
    fail ? 0 : count,
  );
}

export async function lifecycle(count, fail = false, emit = true) {
  // Keep compiler/stats references inside this scope so the caller can GC them
  // after close, rather than counting resources retained by the test itself.
  const { compiler, loaderState } = createCompiler(count, fail, emit);
  try {
    await buildAndCheckLoaders(compiler, loaderState, count, fail);
  } finally {
    await closeCompiler(compiler);
  }
}
