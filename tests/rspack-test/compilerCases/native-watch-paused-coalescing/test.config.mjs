import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { setImmediate, setTimeout } from "node:timers/promises";
import { rspack } from "@rspack/core";

const require = createRequire(import.meta.url);
const aggregateTimeout = 200;
const windows = process.platform === "win32";
const settle = aggregateTimeout + (windows ? 500 : 100);
const observe = windows ? 1500 : 1000;

function deferred() {
  let resolve;
  const promise = new Promise(r => { resolve = r; });
  return { promise, resolve };
}

/** @param {string} file @returns {number} */
function readExecutedBundleValue(file) {
  delete require.cache[require.resolve(file)];
  return require(file).default;
}

async function runArm() {
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
  const rawEventsDuringMakeGate = [];
  const fileInvalidationsDuringMakeGate = [];
  const nodes = ["client", "server"].map(name => ({
    name, runs: [], done: 0, registrations: 0, handle: undefined,
    entered: deferred(), release: deferred(),
  }));
  const waiters = new Set();
  let failure;
  let gated = false;
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
        const run = { changed: new Set(c.modifiedFiles), removed: new Set(c.removedFiles) };
        node.runs.push(run);
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
      c.hooks.invalid.tap("PausedCoalescing", file => {
        if (gated && file) fileInvalidationsDuringMakeGate.push({ node: node.name, file });
      });
    }],
  })));

  // Observe registrations and both raw event surfaces without replacing the
  // callbacks or the graph scheduler. Keep only owned sets and strings.
  compiler.compilers.forEach((c, index) => {
    const node = nodes[index];
    const wfs = c.watchFileSystem;
    const raw = surface => file => {
      if (gated) rawEventsDuringMakeGate.push({ node: node.name, surface, file });
    };
    wfs.on("change", raw("filesystem-change"));
    wfs.on("remove", raw("filesystem-remove"));
    const watch = wfs.watch;
    wfs.watch = function (...args) {
      node.handle = watch.apply(this, args);
      this.watcher.on("change", raw("shim-change"));
      this.watcher.on("remove", raw("shim-remove"));
      node.registrations++;
      pulse();
      return node.handle;
    };
  });

  let watchdog;
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
        gated = true;
        acknowledgement = deferred();
        const acknowledged = acknowledgement.promise;
        fs.writeFileSync(valueFile, "export default 2;");
        await acknowledged;
        await setTimeout(settle, undefined, { signal });
        acknowledgement = undefined;
        gated = false;
        for (const node of nodes) node.release.resolve();
        await waitFor(() => nodes.every(n => n.done >= 3 && n.registrations >= 3));
        await setImmediate(undefined, { signal });
        await setTimeout(observe, undefined, { signal });

        const core = nodes.map(node => ({
          name: node.name,
          runs: node.runs.slice(1),
          value: readExecutedBundleValue(path.join(output, `${node.name}.js`)),
        }));
        for (const node of core) {
          expect(node.runs, `${node.name}: first edit and exactly one paused follow-up`).toHaveLength(2);
          for (const run of node.runs) {
            expect(run.changed.has(valueFile), `${node.name}: filesystem rebuild must include ${valueFile}`).toBe(true);
          }
          expect(node.value, `${node.name}: bundle must include the paused edit`).toBe(2);
        }
        expect(rawEventsDuringMakeGate, "paused make gates must suppress both raw event surfaces").toEqual([]);
        expect(fileInvalidationsDuringMakeGate, "paused make gates must suppress file invalidations").toEqual([]);

        const node = nodes[0];
        const c = compiler.compilers[0];
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
        expect(first.removals.size, "same-value rewrite must not report a removal").toBe(0);
        expect(second.changes.size, "second getInfo must not replay consumed changes").toBe(0);
        expect(second.removals.size, "second getInfo must not replay consumed removals").toBe(0);
        const before = node.runs.length;
        const registrations = node.registrations;
        c.watching.invalidate();
        await waitFor(() => node.done >= before + 1 && node.registrations > registrations);
        await setImmediate(undefined, { signal });
        await setTimeout(observe, undefined, { signal });
        expect(node.runs, "manual invalidation must build once without replaying the drained event").toHaveLength(before + 1);
        expect(readExecutedBundleValue(path.join(output, "client.js")), "manual invalidation must preserve the bundle value").toBe(2);
      })(),
    ]);
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
    await new Promise((resolve, reject) => compiler.close(error => error ? reject(error) : resolve()));
    fs.rmSync(root, { recursive: true, force: true });
  }
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description: "should buffer paused native events into one file-backed follow-up",
  async run() {
    await runArm();
  },
};
