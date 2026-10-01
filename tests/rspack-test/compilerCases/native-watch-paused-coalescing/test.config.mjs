import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { performance } from "node:perf_hooks";
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

async function runArm(nativeWatcher) {
  const arm = nativeWatcher ? "native" : "watchpack";
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
  const started = performance.now();
  const trace = [];
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
  const record = (event, data = {}) => {
    trace.push({ ms: Math.round(performance.now() - started), event, ...data });
  };
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
    experiments: { nativeWatcher },
    watchOptions: {
      aggregateTimeout,
      ignored: file => {
        if (file === valueFile && acknowledgement) {
          record("filter", { node: node.name });
          acknowledgement.resolve();
        }
        return file === output || file.startsWith(`${output}${path.sep}`);
      },
    },
    plugins: [c => {
      c.hooks.watchRun.tap("PausedCoalescing", () => {
        const run = { changed: new Set(c.modifiedFiles), removed: new Set(c.removedFiles) };
        node.runs.push(run);
        record("watchRun", { node: node.name, changed: [...run.changed], removed: [...run.removed] });
        pulse();
      });
      c.hooks.make.tapPromise("PausedCoalescing", async () => {
        if (node.runs.length === 2) {
          record("gate-enter", { node: node.name });
          node.entered.resolve();
          await node.release.promise;
        }
      });
      c.hooks.afterDone.tap("PausedCoalescing", () => {
        node.done++;
        record("done", { node: node.name });
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
      record("rewatch", { node: node.name });
      pulse();
      return node.handle;
    };
  });

  let watchdog;
  try {
    return await Promise.race([
      new Promise((_, reject) => {
        watchdog = globalThis.setTimeout(() => reject(new Error(`${arm}: watchdog expired`)), process.env.CI ? 15000 : 12000);
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
        record("write-1");
        fs.writeFileSync(valueFile, "export default 1;");
        await Promise.all(nodes.map(n => n.entered.promise));
        signal.throwIfAborted();
        gated = true;
        acknowledgement = deferred();
        const acknowledged = acknowledgement.promise;
        record("write-2");
        fs.writeFileSync(valueFile, "export default 2;");
        await acknowledged;
        await setTimeout(settle, undefined, { signal });
        acknowledgement = undefined;
        gated = false;
        record("gate-release");
        for (const node of nodes) node.release.resolve();
        await waitFor(() => nodes.every(n => n.done >= 3 && n.registrations >= 3));
        await setImmediate(undefined, { signal });
        await setTimeout(observe, undefined, { signal });

        const core = nodes.map(node => ({
          name: node.name,
          runs: node.runs.slice(1),
          value: readExecutedBundleValue(path.join(output, `${node.name}.js`)),
        }));
        // Print the core receipt even when the native assertions fail, so the
        // Watchpack arm can run and supply an independent passing control.
        console.log(JSON.stringify({ arm, core, rawEventsDuringMakeGate, fileInvalidationsDuringMakeGate, trace }, (_, value) => value instanceof Set ? [...value] : value));
        for (const node of core) {
          expect(node.runs).toHaveLength(2);
          for (const run of node.runs) {
            expect(run.changed.size + run.removed.size).toBeGreaterThan(0);
            expect(run.changed.has(valueFile)).toBe(true);
          }
          expect(node.value).toBe(2);
        }
        expect(rawEventsDuringMakeGate).toEqual([]);
        expect(fileInvalidationsDuringMakeGate).toEqual([]);

        const node = nodes[0];
        const c = compiler.compilers[0];
        if (nativeWatcher) {
          node.handle.pause();
          acknowledgement = deferred();
          const acknowledged = acknowledgement.promise;
          fs.writeFileSync(valueFile, "export default 2;");
          await acknowledged;
          await setTimeout(settle, undefined, { signal });
          acknowledgement = undefined;
          const first = node.handle.getInfo();
          const second = node.handle.getInfo();
          record("drain", { changed: [...first.changes], removed: [...first.removals], secondChanged: [...second.changes], secondRemoved: [...second.removals] });
          expect(first.changes.has(valueFile)).toBe(true);
          expect(first.removals.size).toBe(0);
          expect(second.changes.size).toBe(0);
          expect(second.removals.size).toBe(0);
        }
        const before = node.runs.length;
        const registrations = node.registrations;
        c.watching.invalidate();
        await waitFor(() => node.done >= before + 1 && node.registrations > registrations);
        await setImmediate(undefined, { signal });
        await setTimeout(observe, undefined, { signal });
        expect(node.runs).toHaveLength(before + 1);
        expect(readExecutedBundleValue(path.join(output, "client.js"))).toBe(2);
        console.log(JSON.stringify({ arm, result: "PASS", trace }));
        return core.map(n => ({ runs: n.runs.length, value: n.value }));
      })(),
    ]);
  } catch (error) {
    console.error(JSON.stringify({ arm, result: "FAIL", message: error.message, trace, rawEventsDuringMakeGate, fileInvalidationsDuringMakeGate }));
    throw error;
  } finally {
    globalThis.clearTimeout(watchdog);
    failure ??= new Error(`${arm}: stopped`);
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
  description: "should buffer paused native events into one file-backed follow-up, like Watchpack",
  async run() {
    const failures = [];
    const results = [];
    for (const native of [true, false]) {
      try { results.push(await runArm(native)); }
      catch (error) { failures.push(error); }
    }
    if (failures.length) throw failures[0];
    expect(results[0]).toEqual(results[1]);
  },
};
