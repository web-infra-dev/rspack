import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { setImmediate, setTimeout } from "node:timers/promises";
import { rspack } from "@rspack/core";

const aggregateTimeout = 200;
const windows = process.platform === "win32";
const settle = aggregateTimeout + (windows ? 500 : 100);
const observe = windows ? 1500 : 1000;

function deferred() {
  let resolve;
  const promise = new Promise(r => { resolve = r; });
  return { promise, resolve };
}

async function runScenario() {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "rspack-watch-")));
  const valueFile = path.join(root, "value.js");
  const output = path.join(root, "dist");
  fs.writeFileSync(path.join(root, "package.json"), JSON.stringify({ type: "commonjs" }));
  fs.writeFileSync(path.join(root, "entry.js"), "import value from './value.js'; export default value;");
  fs.writeFileSync(valueFile, "export default 0;");
  const old = new Date(Date.now() - 10000);
  for (const file of ["package.json", "entry.js", "value.js"]) {
    fs.utimesSync(path.join(root, file), old, old);
  }

  const abort = new AbortController();
  const { signal } = abort;
  const nodes = ["client", "server"].map(name => ({
    name, runs: [], done: 0, registrations: 0, handle: undefined,
    entered: deferred(), release: deferred(),
  }));
  const waiters = new Set();
  let failure;
  let acknowledgement;
  const pulse = () => {
    for (const waiter of waiters) waiter();
  };
  const waitFor = predicate => new Promise((resolve, reject) => {
    const check = () => {
      if (failure) { waiters.delete(check); reject(failure); }
      else if (predicate()) { waiters.delete(check); resolve(); }
    };
    waiters.add(check);
    check();
  });
  const compiler = rspack(nodes.map(node => ({
    name: node.name,
    context: root,
    mode: "development",
    target: node.name === "client" ? "web" : "node",
    devtool: false,
    entry: "./entry.js",
    module: { rules: [{ test: /\.js$/, type: "javascript/auto" }] },
    output: { path: output, filename: `${node.name}.js`, library: { type: "commonjs2" } },
    experiments: { nativeWatcher: true },
    watchOptions: {
      aggregateTimeout,
      ignored: file => {
        if (file === valueFile && acknowledgement) {
          acknowledgement.resolve();
        }
        return file === output || file.startsWith(`${output}${path.sep}`);
      },
    },
    plugins: [c => {
      c.hooks.watchRun.tap("PausedCoalescing", () => {
        node.runs.push(true);
        pulse();
      });
      c.hooks.make.tapPromise("PausedCoalescing", async () => {
        if (node.runs.length === 2) {
          node.entered.resolve();
          await node.release.promise;
        }
      });
      c.hooks.afterDone.tap("PausedCoalescing", () => {
        node.done++;
        pulse();
      });
    }],
  })));

  // Capture watch handles and registrations without replacing callbacks or the graph scheduler.
  compiler.compilers.forEach((c, index) => {
    const node = nodes[index];
    const wfs = c.watchFileSystem;
    const watch = wfs.watch;
    wfs.watch = function (...args) {
      node.handle = watch.apply(this, args);
      node.registrations++;
      pulse();
      return node.handle;
    };
  });

  let watchdog;
  let succeeded = false;
  try {
    await Promise.race([
      new Promise((_, reject) => {
        watchdog = globalThis.setTimeout(() => reject(new Error("native watcher: paused coalescing watchdog expired")), process.env.CI ? 15000 : 12000);
      }),
      (async () => {
        compiler.watch(compiler.compilers.map(c => c.options.watchOptions), (error, stats) => {
          if (error || stats?.hasErrors()) {
            failure = error || new Error(stats.toString());
            pulse();
          }
        });
        await waitFor(() => nodes.every(n => n.done >= 1 && n.registrations >= 1));
        await setImmediate(undefined, { signal });
        await setTimeout(windows ? 500 : 250, undefined, { signal });
        fs.writeFileSync(valueFile, "export default 1;");
        await Promise.all(nodes.map(n => n.entered.promise));
        signal.throwIfAborted();
        acknowledgement = deferred();
        const acknowledged = acknowledgement.promise;
        fs.writeFileSync(valueFile, "export default 2;");
        await acknowledged;
        await setTimeout(settle, undefined, { signal });
        acknowledgement = undefined;
        for (const node of nodes) node.release.resolve();
        await waitFor(() => nodes.every(n => n.done >= 3 && n.registrations >= 3));
        await setImmediate(undefined, { signal });
        await setTimeout(observe, undefined, { signal });

        for (const node of nodes) {
          expect(node.runs.slice(1), `${node.name}: first edit and exactly one paused follow-up`).toHaveLength(2);
        }

        const node = nodes[0];
        node.handle.pause();
        acknowledgement = deferred();
        const drainAcknowledged = acknowledgement.promise;
        fs.writeFileSync(valueFile, "export default 2;");
        await drainAcknowledged;
        await setTimeout(settle, undefined, { signal });
        acknowledgement = undefined;
        const first = node.handle.getInfo();
        const second = node.handle.getInfo();
        expect(first.changes.has(valueFile), "first getInfo must consume the paused change").toBe(true);
        expect({ changes: second.changes.size, removals: second.removals.size }, "second getInfo must return empty sets").toEqual({ changes: 0, removals: 0 });
      })(),
    ]);
    succeeded = true;
  } finally {
    globalThis.clearTimeout(watchdog);
    failure ??= new Error("native watcher: stopped");
    abort.abort(failure);
    acknowledgement?.resolve();
    for (const node of nodes) {
      node.entered.resolve();
      node.release.resolve();
    }
    pulse();
    let cleanupError;
    try {
      await new Promise((resolve, reject) => compiler.close(error => error ? reject(error) : resolve()));
    } catch (error) {
      cleanupError = error;
    } finally {
      try {
        fs.rmSync(root, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
      } catch (error) {
        cleanupError ??= error;
      }
    }
    if (succeeded && cleanupError) throw cleanupError;
  }
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description: "should coalesce paused native saves and consume pending changes",
  async run() {
    await runScenario();
  },
};
