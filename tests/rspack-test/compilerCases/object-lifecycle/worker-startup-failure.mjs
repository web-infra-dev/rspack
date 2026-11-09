import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { rspack } from "@rspack/core";
import { createFsFromVolume, Volume } from "memfs";
import { closeCompiler, runCompiler } from "./helpers.mjs";

const require = createRequire(import.meta.url);
const threads = require("node:worker_threads");
const OriginalWorker = threads.Worker;
const dynamic = process.argv.includes("--dynamic");
threads.Worker = class extends OriginalWorker {
  constructor() {
    super('throw new Error("intentional loader worker startup failure")', { eval: true });
  }
};

async function build(parallel) {
  const use = [{
    loader: require.resolve("./subscription-loader.cjs"),
    options: { name: "startup", parallel },
    parallel: parallel ? { maxWorkers: 1 } : false,
  }];
  const compiler = rspack({
    context: import.meta.dirname,
    mode: "development",
    entry: "./entry.js",
    // Dynamic use starts the pool after make, so the runner queues native work
    // before startup fails instead of only rejecting the make readiness wait.
    module: { rules: [{ test: /entry\.js$/, use: dynamic ? () => use : use }] },
    name: "startup",
    output: { path: "/", filename: "bundle.js" },
  });
  compiler.outputFileSystem = createFsFromVolume(new Volume());
  try {
    const stats = await runCompiler(compiler);
    if (stats.hasErrors()) throw new Error(stats.toString());
  } finally {
    await closeCompiler(compiler);
  }
}

try {
  await assert.rejects(build(true), dynamic
    ? /No parallel loader workers are available/
    : /intentional loader worker startup failure/);
} finally {
  threads.Worker = OriginalWorker;
}
// A failed worker pool must not prevent main-only compilers from consuming tasks.
await build(false);
