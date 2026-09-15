const path = require("path");

const options = () => ["a", "b"].map(name => ({
  name,
  mode: "development",
  context: path.resolve(__dirname, "../fixtures"),
  entry: `./${name}.js`
}));
const close = compiler => new Promise((resolve, reject) => {
  compiler.close(error => error ? reject(error) : resolve());
});

/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig[]} */
module.exports = [
  ...[Infinity, 1].map(parallelism => ({
    description: `should wait for outdated rebuilds with parallelism ${parallelism}`,
    options: () => Object.assign(options(), { parallelism }),
    async build(context, compiler) {
      const publications = [];
      const generations = [0, 0];
      const completed = [0, 0];
      const gates = [new Map(), new Map()];
      const waiters = new Set();
      const signal = () => { for (const check of [...waiters]) check(); };
      const until = condition => new Promise(resolve => {
        const check = () => {
          if (condition()) { waiters.delete(check); resolve(); }
        };
        waiters.add(check);
        check();
      });
      const snapshots = new WeakMap();
      compiler.compilers.forEach((child, index) => {
        child.hooks.watchRun.tap("Trace", () => { generations[index]++; });
        child.hooks.make.tapAsync("Gate", (compilation, callback) => {
          snapshots.set(compilation, generations[index]);
          if (generations[index] === 1) return callback();
          gates[index].set(generations[index], () => {
            if (callback) {
              const release = callback;
              callback = undefined;
              release();
            }
          });
          signal();
        });
        child.hooks.done.tap("Trace", stats => {
          completed[index] = generations[index];
          signal();
        });
      });
      compiler.hooks.done.tap("Capture", stats => {
        publications.push(stats.stats.map(stat => snapshots.get(stat.compilation)));
      });
      let callbacks = 0;
      let fail;
      const failed = new Promise((_, reject) => { fail = reject; });
      const watching = compiler.watch({}, error => {
        if (error) return fail(error);
        callbacks++;
        signal();
      });
      const scenario = async () => {
        await until(() => callbacks === 1);
        watching.invalidate();
        await until(() => gates[0].has(2));
        if (parallelism !== 1) await until(() => gates[1].has(2));
        compiler.compilers[0].watching.invalidate();
        gates[0].get(2)();
        await until(() => gates[1].has(2));
        if (parallelism !== 1) await until(() => gates[0].has(3));
        gates[1].get(2)();
        await until(() => completed[1] === 2);
        // Let both completion hooks and the graph queue finish processing.
        await new Promise(resolve => setImmediate(resolve));
        expect(publications).toEqual([[1, 1]]);
        expect(callbacks).toBe(1);
        await until(() => gates[0].has(3));
        gates[0].get(3)();
        await until(() => callbacks === 2);
        expect(publications).toEqual([[1, 1], [3, 2]]);
      };
      try {
        await Promise.race([scenario(), failed]);
      } finally {
        // Release any in-flight compilation before closing on assertion failure.
        for (const entries of gates) {
          for (const release of entries.values()) release();
        }
        await close(compiler);
      }
    }
  })),
  ...[false, true].map(dependent => ({
    description: `should retain invalidations from aggregate done with dependencies ${dependent}`,
    options: () => {
      const configs = options();
      if (dependent) configs[1].dependencies = ["a"];
      return configs;
    },
    async build(context, compiler) {
      const publications = [];
      const callbacks = [];
      const generations = [0, 0];
      const snapshots = new WeakMap();
      compiler.compilers.forEach((child, index) => {
        child.hooks.watchRun.tap("Trace", () => { generations[index]++; });
        child.hooks.done.tapAsync("Trace", (stats, callback) => {
          setImmediate(() => {
            snapshots.set(stats, generations[index]);
            callback();
          });
        });
      });
      compiler.hooks.done.tap("Capture", stats => {
        publications.push(stats.stats.map(stat => snapshots.get(stat)));
        if (publications.length === 1) compiler.compilers[0].watching.invalidate();
      });
      try {
        await new Promise((resolve, reject) => {
          compiler.watch({}, (error, stats) => {
            if (error) return reject(error);
            callbacks.push(stats.stats.map(stat => stat.compilation.name));
            if (callbacks.length === 2) resolve();
          });
        });
        expect(publications).toEqual([[1, 1], [2, dependent ? 2 : 1]]);
        expect(callbacks).toEqual([["a", "b"], dependent ? ["a", "b"] : ["a"]]);
      } finally {
        await close(compiler);
      }
    }
  })),
  {
    description: "should publish once per run when reusing a multi compiler",
    options,
    async build(context, compiler) {
      const publications = [];
      compiler.hooks.done.tap("Capture", stats => {
        publications.push(stats.stats.map(stat => stat.compilation.name));
      });
      try {
        for (let i = 0; i < 2; i++) {
          await new Promise((resolve, reject) => {
            compiler.run(error => error ? reject(error) : resolve());
          });
          expect(publications).toHaveLength(i + 1);
        }
        expect(publications).toEqual([["a", "b"], ["a", "b"]]);
      } finally {
        await close(compiler);
      }
    }
  },
  {
    description: "should recover from watchRun errors and publish after closing and rewatching",
    options,
    async build(context, compiler) {
      const publications = [];
      const failure = new Error("watchRun failure");
      let shouldFail = true;
      compiler.compilers[0].hooks.watchRun.tapAsync("FailOnce", (_, callback) => {
        callback(shouldFail ? failure : null);
      });
      compiler.hooks.done.tap("Capture", stats => { publications.push(stats); });
      const stopWatching = () => new Promise((resolve, reject) => {
        compiler.watching.close(error => error ? reject(error) : resolve());
      });
      try {
        const error = await new Promise(resolve => {
          compiler.watch({}, error => resolve(error));
        });
        expect(error).toBe(failure);
        expect(publications).toHaveLength(0);
        await stopWatching();
        shouldFail = false;
        for (let i = 0; i < 2; i++) {
          await new Promise((resolve, reject) => {
            compiler.watch({}, error => error ? reject(error) : resolve());
          });
          expect(publications).toHaveLength(i + 1);
          expect(publications[i].stats.map(stat => stat.compilation.name)).toEqual(["a", "b"]);
          await stopWatching();
        }
        expect(publications[0].stats[0]).not.toBe(publications[1].stats[0]);
      } finally {
        await close(compiler);
      }
    }
  },
  ...[false, true].map(watch => ({
    description: `should report aggregate done errors through the ${watch ? "watch" : "run"} callback`,
    options,
    async build(context, compiler) {
      const failure = new Error("aggregate done failure");
      compiler.hooks.done.tap("Fail", () => { throw failure; });
      try {
        const error = await new Promise(resolve => {
          if (watch) compiler.watch({}, error => resolve(error));
          else compiler.run(error => resolve(error));
        });
        expect(error).toBe(failure);
      } finally {
        await close(compiler);
      }
    }
  }))
];
