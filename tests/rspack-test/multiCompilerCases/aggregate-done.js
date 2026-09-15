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
        child.hooks.done.tapAsync({ name: "Trace", stage: -1 }, (stats, callback) => {
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
            resolve();
          });
        });
        expect(publications).toEqual([[1, 1], [2, dependent ? 2 : 1]]);
        expect(callbacks).toEqual([["a", "b"]]);
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
    description: "should aggregate independent runs after watch setup fails on busy children",
    options,
    async build(context, compiler) {
      const events = [];
      const releases = [];
      const entered = compiler.compilers.map((child, index) => new Promise(resolve => {
        let first = true;
        child.hooks.make.tapAsync("Gate", (_, callback) => {
          if (!first) return callback();
          first = false;
          releases[index] = () => {
            if (callback) {
              const release = callback;
              callback = undefined;
              release();
            }
          };
          resolve();
        });
        child.hooks.done.tap("Trace", () => { events.push(`${child.name}.done`); });
        child.hooks.afterDone.tap("Trace", () => { events.push(`${child.name}.afterDone`); });
      }));
      compiler.hooks.done.tap("Trace", () => { events.push("parent.done"); });
      const run = child => new Promise((resolve, reject) => {
        child.run(error => {
          events.push(`${child.name}.callback`);
          error ? reject(error) : resolve();
        });
      });
      const initial = compiler.compilers.map(run);
      try {
        await Promise.all(entered);
        const error = await new Promise(resolve => compiler.watch({}, resolve));
        expect(error.name).toBe("ConcurrentCompilationError");
        expect(compiler.compilers.every(child => child.watching === undefined)).toBe(true);
        events.push("watch.error");
        for (let i = 0; i < initial.length; i++) {
          releases[i]();
          await initial[i];
        }
        await run(compiler.compilers[0]);
        expect(events).toEqual([
          "watch.error", "a.done", "a.callback", "a.afterDone",
          "parent.done", "b.done", "b.callback", "b.afterDone",
          "parent.done", "a.done", "a.callback", "a.afterDone"
        ]);
      } finally {
        for (const release of releases) release();
        await Promise.all(initial);
        await close(compiler);
      }
    }
  },
  ...[0, 1].map(busyIndex => ({
    description: `should release a failed watch graph when child ${busyIndex} is busy`,
    options,
    async build(context, compiler) {
      const events = [];
      compiler.hooks.done.tap("Trace", () => { events.push("parent.done"); });
      const child = compiler.compilers[busyIndex];
      child.hooks.done.tap("Trace", () => { events.push("child.done"); });
      child.hooks.afterDone.tap("Trace", () => { events.push("child.afterDone"); });
      let release;
      let initial;
      try {
        await new Promise((resolve, reject) => compiler.run(error => error ? reject(error) : resolve()));
        events.length = 0;
        let enter;
        const entered = new Promise(resolve => { enter = resolve; });
        child.hooks.make.tapAsync("Gate", (_, callback) => {
          release = () => {
            if (callback) {
              const finish = callback;
              callback = undefined;
              finish();
            }
          };
          enter();
        });
        initial = new Promise((resolve, reject) => child.run(error => {
          events.push("child.callback");
          error ? reject(error) : resolve();
        }));
        await entered;
        const error = await new Promise(resolve => compiler.watch({}, resolve));
        expect(error.name).toBe("ConcurrentCompilationError");
        events.push("watch.error");
        // The first setup can fail before a later idle child gets a watcher.
        // That remaining watcher must not keep an already failed graph active.
        if (busyIndex === 0) expect(compiler.compilers[1].watching).toBeDefined();
        release();
        await initial;
        expect(events).toEqual([
          "watch.error", "parent.done", "child.done", "child.callback", "child.afterDone"
        ]);
      } finally {
        release?.();
        if (initial) await initial;
        await close(compiler);
      }
    }
  })),
  {
    description: "should retain completion of a closed dependent watcher across rebuilds",
    options: () => {
      const configs = options();
      configs[1].dependencies = ["a"];
      return configs;
    },
    async build(context, compiler) {
      const events = [];
      const [a, b] = compiler.compilers;
      compiler.hooks.done.tap("Trace", () => { events.push("parent.done"); });
      a.hooks.done.tap("Trace", () => { events.push("a.done"); });
      a.hooks.afterDone.tap("Trace", () => { events.push("a.afterDone"); });
      try {
        await new Promise((resolve, reject) => compiler.watch({}, error => error ? reject(error) : resolve()));
        await new Promise((resolve, reject) => b.watching.close(error => error ? reject(error) : resolve()));
        for (let round = 0; round < 3; round++) {
          events.length = 0;
          await new Promise(resolve => {
            a.hooks.afterDone.tap(`Round${round}`, resolve);
            a.watching.invalidate();
          });
          // Let the graph attempt to start the detached dependent before the next round.
          await new Promise(resolve => setImmediate(resolve));
          expect(events).toEqual(["parent.done", "a.done", "a.afterDone"]);
        }
      } finally {
        await close(compiler);
      }
    }
  },
  {
    description: "should ignore a closed outdated watcher while other watchers remain active",
    options,
    async build(context, compiler) {
      const events = [];
      const [a, b] = compiler.compilers;
      compiler.hooks.done.tap("Trace", () => { events.push("parent.done"); });
      a.hooks.done.tap("Trace", () => { events.push("a.done"); });
      a.hooks.afterDone.tap("Trace", () => { events.push("a.afterDone"); });
      let release;
      try {
        await new Promise((resolve, reject) => compiler.watch({}, error => error ? reject(error) : resolve()));
        events.length = 0;
        let enter;
        const entered = new Promise(resolve => { enter = resolve; });
        let first = true;
        a.hooks.make.tapAsync("Gate", (_, callback) => {
          if (!first) return callback();
          first = false;
          release = () => {
            if (callback) {
              const finish = callback;
              callback = undefined;
              finish();
            }
          };
          enter();
        });
        a.watching.invalidate();
        await entered;
        a.watching.invalidate();
        const closed = new Promise(resolve => a.watching.close(resolve));
        release();
        await closed;
        expect(b.watching).toBeDefined();
        await new Promise((resolve, reject) => a.run(error => {
          events.push("a.callback");
          error ? reject(error) : resolve();
        }));
        expect(events).toEqual(["parent.done", "a.done", "a.callback", "a.afterDone"]);
        events.length = 0;
        let enterB;
        const enteredB = new Promise(resolve => { enterB = resolve; });
        let finishB;
        const finishedB = new Promise(resolve => { finishB = resolve; });
        let builds = 0;
        b.hooks.make.tapAsync("Gate", (_, callback) => {
          builds++;
          if (builds !== 1) return callback();
          release = () => {
            if (callback) {
              const finish = callback;
              callback = undefined;
              finish();
            }
          };
          enterB();
        });
        b.hooks.done.tap("Trace", () => { events.push(`b.done:${builds}`); });
        b.hooks.afterDone.tap("Trace", () => { if (builds === 2) finishB(); });
        b.watching.invalidate();
        await enteredB;
        b.watching.invalidate();
        release();
        await finishedB;
        // Ignoring closed a must not bypass b's still-active outdated build.
        expect(events).toEqual(["b.done:1", "parent.done", "b.done:2"]);
      } finally {
        release?.();
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
      compiler.hooks.done.tap("Capture", stats => { publications.push([...stats.stats]); });
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
          expect(publications[i].map(stat => stat.compilation.name)).toEqual(["a", "b"]);
          await stopWatching();
        }
        expect(publications[0][0]).not.toBe(publications[1][0]);
      } finally {
        await close(compiler);
      }
    }
  },
  {
    description: "should preserve aggregate notification before closing during asynchronous child done",
    options: () => {
      const configs = options();
      configs[0].dependencies = ["b"];
      return configs;
    },
    async build(context, compiler) {
      let release;
      let entered;
      const waiting = new Promise(resolve => { entered = resolve; });
      compiler.compilers[0].hooks.done.tapAsync("Gate", (_, callback) => {
        release = callback;
        entered();
      });
      const publications = [];
      compiler.hooks.done.tap("Capture", stats => publications.push(stats));
      let finish;
      let fail;
      const finished = new Promise((resolve, reject) => { finish = resolve; fail = reject; });
      compiler.watch({}, error => error ? fail(error) : finish());
      try {
        await waiting;
        await new Promise(resolve => compiler.watching.close(resolve));
        release();
        await finished;
        expect(publications).toHaveLength(1);
      } finally {
        await close(compiler);
      }
    }
  },
  ...[false, true].flatMap(watch => [false, true].map(throws => ({
    description: `should preserve interleaved done hooks in ${watch ? "watch" : "run"} with aggregate failure ${throws}`,
    options,
    async build(context, compiler) {
      const events = [];
      const failure = new Error("aggregate failure");
      const [a, b] = compiler.compilers;
      let enter;
      const entered = new Promise(resolve => { enter = resolve; });
      let release;
      let finishA;
      const finishedA = new Promise(resolve => { finishA = resolve; });
      a.hooks.done.tapAsync("Gate", (_, callback) => {
        events.push("a.done:start");
        release = () => { events.push("a.done:end"); callback(); };
        enter();
      });
      b.hooks.make.tapAsync("Gate", (_, callback) => {
        entered.then(() => callback());
      });
      b.hooks.done.tap("Trace", () => { events.push("b.done"); });
      for (const child of compiler.compilers) {
        child.hooks.failed.tap("Trace", error => {
          events.push(`${child.name}.failed`);
          if (child === b) setImmediate(release);
        });
        child.hooks.afterDone.tap("Trace", stats => {
          events.push(`${child.name}.afterDone:${Boolean(stats)}`);
          if (child === b && stats) setImmediate(release);
          if (child === a) finishA();
        });
      }
      compiler.hooks.done.tap("Trace", () => {
        events.push("parent.done");
        if (throws) throw failure;
      });
      try {
        const error = await new Promise(resolve => {
          const callback = error => {
            events.push(`callback:${error ? "error" : "success"}`);
            resolve(error);
          };
          if (watch) compiler.watch({}, callback);
          else compiler.run(callback);
        });
        await finishedA;
        expect(error).toBe(throws ? failure : null);
        expect(events).toEqual(throws ? [
          "a.done:start", "parent.done", "b.failed", "callback:error",
          ...(!watch ? ["b.afterDone:false"] : []),
          "a.done:end", "a.afterDone:true"
        ] : [
          "a.done:start", "parent.done", "b.done", "b.afterDone:true",
          "a.done:end", "a.afterDone:true", "callback:success"
        ]);
      } finally {
        await close(compiler);
      }
    }
  }))),
  ...[false, true].map(watch => ({
    description: `should aggregate independently started children in ${watch ? "watch" : "run"}`,
    options,
    async build(context, compiler) {
      const events = [];
      compiler.hooks.done.tap("Trace", () => { events.push("parent.done"); });
      compiler.compilers.forEach(child => {
        child.hooks.done.tap("Trace", () => { events.push(`${child.name}.done`); });
        child.hooks.afterDone.tap("Trace", () => { events.push(`${child.name}.afterDone`); });
      });
      try {
        for (const child of compiler.compilers) {
          await new Promise((resolve, reject) => {
            const callback = error => {
              events.push(`${child.name}.callback`);
              error ? reject(error) : resolve();
            };
            if (watch) child.watch({}, callback);
            else child.run(callback);
          });
        }
        expect(events).toEqual([
          "a.done", "a.callback", "a.afterDone",
          "parent.done", "b.done", "b.callback", "b.afterDone"
        ]);
      } finally {
        await close(compiler);
      }
    }
  })),
  ...[false, true].flatMap(watch => [false, true].map(throws => ({
    description: `should preserve completion hook order in ${watch ? "watch" : "run"} with aggregate failure ${throws}`,
    options: () => {
      const configs = options();
      configs[0].dependencies = ["b"];
      return configs;
    },
    async build(context, compiler) {
      const failure = new Error("aggregate done failure");
      const events = [];
      compiler.compilers.forEach(child => {
        child.hooks.done.tapAsync({ name: "Capture", stage: -1 }, (_, callback) => {
          events.push(["done:start", child.name]);
          setImmediate(() => {
            events.push(["done:end", child.name]);
            callback();
          });
        });
        child.hooks.failed.tap("Capture", error => {
          events.push(["failed", child.name, error]);
        });
        child.hooks.afterDone.tap("Capture", stats => {
          events.push(["afterDone", child.name, Boolean(stats)]);
        });
      });
      compiler.hooks.done.tap("Capture", () => {
        events.push(["aggregate done"]);
        if (throws) throw failure;
      });
      try {
        const error = await new Promise(resolve => {
          const callback = error => {
            events.push(["callback", error]);
            resolve(error);
          };
          if (watch) compiler.watch({}, callback);
          else compiler.run(callback);
        });
        expect(error).toBe(throws ? failure : null);
        const expected = [
          ["done:start", "b"],
          ["done:end", "b"],
          ["afterDone", "b", true],
          ["done:start", "a"],
          ["done:end", "a"],
          ["aggregate done"]
        ];
        if (throws) {
          expected.push(["failed", "a", failure], ["callback", failure]);
          if (!watch) expected.push(["afterDone", "a", false]);
        } else {
          // Successful multi callbacks are queued; the child's afterDone is not.
          expected.push(["afterDone", "a", true], ["callback", null]);
        }
        expect(events).toEqual(expected);
      } finally {
        await close(compiler);
      }
    }
  })))
];
