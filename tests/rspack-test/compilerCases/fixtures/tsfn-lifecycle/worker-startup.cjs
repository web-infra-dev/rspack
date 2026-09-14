const assert = require("node:assert/strict");
const path = require("node:path");
const { setImmediate } = require("node:timers/promises");
const threads = require("node:worker_threads");
const { createFsFromVolume, Volume } = require("memfs");
const { closeCompiler, runCompiler } = require("./helpers.cjs");

module.exports = async function main(parallel = false) {
  process.env.RSPACK_LOADER_WORKER_THREADS = "2";
  const workers = [];
  const OriginalWorker = threads.Worker;
  let open = false;
  threads.Worker = class extends OriginalWorker {
    constructor(...args) {
      super(...args);
      this.stopped = false;
      this.observedReady = new Promise(resolve => { this.onReady = resolve; });
      workers.push(this);
    }
    emit(name, message, ...args) {
      if (name === "message" && message?.type === "rspack-loader-worker-ready") {
        this.onReady();
        if (!open) { this.readyMessage = message; return true; }
      }
      return super.emit(name, message, ...args);
    }
    terminate() { this.stopped = true; return super.terminate(); }
    release() {
      if (!this.stopped) {
        assert(this.readyMessage);
        super.emit("message", this.readyMessage);
      }
    }
  };
  const { rspack, workerFunction } = require("@rspack/core");
  // Pool creation must happen at import, before any compiler exists.
  assert.equal(workers.length, 2);
  const create = extra => {
    const compiler = rspack({ context: __dirname, mode: "development", cache: false,
      devtool: false, entry: "./entry.js", output: { path: "/out" }, ...extra });
    compiler.outputFileSystem = createFsFromVolume(new Volume());
    return compiler;
  };
  // Ordinary builds do not wait for this unused pool's withheld ready messages.
  const ordinary = create({});
  try { assert(!(await runCompiler(ordinary)).hasErrors()); }
  finally { await closeCompiler(ordinary); }
  const extra = parallel ? { module: { rules: [{ test: /entry\.js$/, use: [{
    loader: path.join(__dirname, "parallel-loader.cjs"), parallel: { maxWorkers: 1 },
  }] }] } } : { optimization: { minimize: false, splitChunks: {
    chunks: "all", minSize: 0,
    name: workerFunction(path.join(__dirname, "../worker-function-split-chunks/name.cjs"), { name: "shared" }),
    cacheGroups: { default: false, defaultVendors: false, shared: { test: /entry\.js$/, enforce: true } },
  } } };
  const compiler = create(extra);
  let made = false;
  compiler.hooks.make.tap("Readiness", () => { assert(open); made = true; });
  try {
    const build = runCompiler(compiler);
    // Attach immediately so a failed build is never an unhandled rejection.
    const outcome = build.then(stats => ({ stats }), error => ({ error }));
    await setImmediate();
    const active = workers.filter(worker => !worker.stopped);
    assert.equal(active.length, parallel ? 1 : 2);
    await Promise.all(active.map(worker => worker.observedReady));
    await setImmediate();
    assert.equal(made, false);
    open = true;
    for (const worker of active) worker.release();
    const result = await outcome;
    if (result.error) throw result.error;
    assert(!result.stats.hasErrors(), result.stats.toString());
    assert(made);
    assert.equal(workers.length, 2, "Import must not recursively spawn worker pools");
  } finally { await closeCompiler(compiler); }
  console.log("worker-startup-complete");
};

if (require.main === module) module.exports().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
