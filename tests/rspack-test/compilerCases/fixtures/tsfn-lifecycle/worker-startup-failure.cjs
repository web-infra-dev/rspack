const assert = require("node:assert/strict");
const path = require("node:path");
const threads = require("node:worker_threads");
const { closeCompiler, runCompiler } = require("./helpers.cjs");
process.env.RSPACK_LOADER_WORKER_THREADS = "1";
const OriginalWorker = threads.Worker;
threads.Worker = class extends OriginalWorker {
  constructor(_filename, options) {
    super(path.join(__dirname, "missing-startup-worker.cjs"), options);
  }
};
const { rspack, workerFunction } = require("@rspack/core");
const compiler = rspack({
  context: __dirname, mode: "development", cache: false, entry: "./entry.js",
  optimization: { splitChunks: { name: workerFunction(path.join(__dirname,
    "../worker-function-split-chunks/name.cjs"), { name: "shared" }) } },
});
(async () => {
  try { await assert.rejects(runCompiler(compiler), /missing-startup-worker/); }
  finally { await closeCompiler(compiler); }
  console.log("worker-startup-failure-complete");
})().catch(error => { console.error(error); process.exitCode = 1; });
