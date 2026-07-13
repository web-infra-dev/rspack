const { createFsFromVolume, Volume } = require("memfs");
const { lazyCompilationMiddleware } = require("@rspack/core");
const path = require("node:path");
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
      lazyCompilation: {
        entries: false,
        imports: false,
      },
    };
  },
  async compiler(context, compiler) {
    compiler.outputFileSystem = createFsFromVolume(new Volume());
  },
  async build(context, compiler) {
    const cycles = [];
    let coalesceNormalInvalidation = false;
    let emitNormalFileChange;
    let watching;
    compiler.watchFileSystem = {
      watch(
        files,
        dirs,
        missing,
        startTime,
        options,
        callback,
        callbackUndelayed,
      ) {
        emitNormalFileChange = () => {
          const file = path.join(compiler.context, "c.js");
          callbackUndelayed(file, Date.now());
          callback(null, new Map(), new Map(), new Set([file]), new Set());
        };
        return {
          close() {},
          getInfo() {
            return {
              changes: new Set(),
              removals: new Set(),
              fileTimeInfoEntries: new Map(),
              contextTimeInfoEntries: new Map(),
            };
          },
          pause() {},
        };
      },
    };
    compiler.hooks.thisCompilation.tap(
      "test lazy invalidation provenance",
      compilation => {
        cycles.push(compilation.watchInvalidationKind);
      },
    );
    compiler.hooks.make.tapAsync(
      "coalesce normal invalidation",
      (compilation, callback) => {
        if (
          coalesceNormalInvalidation &&
          compilation.watchInvalidationKind === "lazy"
        ) {
          coalesceNormalInvalidation = false;
          emitNormalFileChange();
        }
        callback();
      },
    );

    const middleware = lazyCompilationMiddleware(compiler);
    const activate = module =>
      new Promise((resolve, reject) => {
        middleware(
          {
            body: [module],
            method: "POST",
            url: "/_rspack/lazy/trigger",
          },
          {
            end: resolve,
            write() {},
            writeHead(status) {
              expect(status).toBe(200);
            },
          },
          reject,
        ).catch(reject);
      });

    return new Promise((resolve, reject) => {
      let builds = 0;
      watching = compiler.watch({}, err => {
        if (err) return reject(err);
        builds++;
        if (builds === 1) {
          setImmediate(() => activate("first-lazy-module").catch(reject));
          return;
        }
        if (builds === 2) {
          coalesceNormalInvalidation = true;
          setImmediate(() => activate("coalesced-lazy-module").catch(reject));
          return;
        }
        if (builds === 3) {
          try {
            expect(cycles).toEqual([undefined, "lazy", "lazy", "normal"]);
          } catch (error) {
            watching.close(() => reject(error));
            return;
          }
          watching.close(error => (error ? reject(error) : resolve()));
        }
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
