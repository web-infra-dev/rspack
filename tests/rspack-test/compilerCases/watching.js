const { createFsFromVolume, Volume } = require("memfs");
const path = require("path");
const { start } = require("@rspack/test-tools/helper/legacy/deprecationTracking");
let tracker = null;

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [{
  description: "should set compiler.watching correctly",
  options(context) {
    return {
      entry: "./c",
    };
  },
  async compiler(context, compiler) {
    compiler.outputFileSystem = createFsFromVolume(new Volume());
  },
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      const watching = compiler.watch({}, (err, stats) => {
        if (err) return reject(err);
        watching.close(resolve);
      });
      expect(compiler.watching).toBe(watching);
    });
  },
}, {
  description: "should flag watchMode as true in watch",
  options(context) {
    return {
      entry: "./c",
    };
  },
  async compiler(context, compiler) {
    compiler.outputFileSystem = createFsFromVolume(new Volume());
  },
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      const watch = compiler.watch({}, err => {
        if (err) return reject(err);
        expect(compiler.watchMode).toBeTruthy();
        watch.close(() => {
          expect(compiler.watchMode).toBeFalsy();
          resolve();
        });
      });
    });
  },
}, {
  description: "should snapshot lazy compilation invalidation provenance per watch cycle",
  options(context) {
    return {
      entry: "./c",
    };
  },
  async compiler(context, compiler) {
    compiler.outputFileSystem = createFsFromVolume(new Volume());
  },
  async build(context, compiler) {
    const current = Symbol.for("rspack.lazyCompilationCurrent");
    const invalidation = Symbol.for("rspack.lazyCompilationInvalidation");
    const cycles = [];
    compiler.hooks.thisCompilation.tap("test lazy invalidation provenance", () => {
      cycles.push(compiler[current] === true);
    });

    return new Promise((resolve, reject) => {
      let builds = 0;
      const watching = compiler.watch({}, err => {
        if (err) return reject(err);
        builds++;
        if (builds === 1) {
          compiler[invalidation] = true;
          watching.invalidate();
          return;
        }
        if (builds === 2) {
          watching.invalidate();
          return;
        }
        if (builds === 3) {
          compiler[invalidation] = true;
          watching.invalidate();
          return;
        }
        if (builds === 4) {
          compiler[invalidation] = true;
          watching.invalidate();
          return;
        }
        if (builds === 5) {
          const getInfo = watching.watcher.getInfo;
          watching.watcher.getInfo = () => ({
            ...getInfo(),
            changes: new Set([path.join(compiler.context, "c.js")]),
          });
          compiler[invalidation] = true;
          watching.invalidate();
          return;
        }
        if (builds === 6) {
          expect(cycles).toEqual([false, true, false, true, true, false]);
          watching.close(resolve);
        }
      });
    });
  },
}, {
  description: "should downgrade lazy provenance for pending native watcher changes",
  options(context) {
    return {
      entry: "./c",
      experiments: {
        nativeWatcher: true,
      },
    };
  },
  async compiler(context, compiler) {
    compiler.outputFileSystem = createFsFromVolume(new Volume());
  },
  async build(context, compiler) {
    const current = Symbol.for("rspack.lazyCompilationCurrent");
    const invalidation = Symbol.for("rspack.lazyCompilationInvalidation");
    const cycles = [];
    compiler.hooks.thisCompilation.tap("test native lazy invalidation provenance", () => {
      cycles.push(compiler[current] === true);
    });

    return new Promise((resolve, reject) => {
      let builds = 0;
      const watching = compiler.watch({}, err => {
        if (err) return reject(err);
        builds++;
        if (builds === 1) {
          compiler[invalidation] = true;
          watching.watcher._onChange(path.join(compiler.context, "c.js"));
          return;
        }
        expect(cycles).toEqual([false, false]);
        watching.close(resolve);
      });
    });
  },
}, {
  description: "should deprecate when watch option is used without callback",
  options(context) {
    tracker = start();
    return {
      watch: true
    };
  },
  async compiler(context, compiler) {
    compiler.outputFileSystem = createFsFromVolume(new Volume());
  },
  async build(context, compiler) {

  },
  async check() {
    const deprecations = tracker();
    expect(deprecations).toHaveLength(1);
    expect(deprecations[0].message).toContain("A 'callback' argument needs to be provided");
  }
}];
