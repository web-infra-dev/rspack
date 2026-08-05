const { LazyCompilationTestPlugin } = require("@rspack/test-tools");
const { lazyCompilationMiddleware } = require("@rspack/core");
const path = require("path");
const context = path.join(__dirname, "../fixtures");

/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig[]} */
module.exports = [{
  description: "compiler has unique lazy compilation config",
  options() {
    return [
      {
        entry: "./esm/a.js",
        context,
        lazyCompilation: false,
      },
      {
        entry: "./esm/b.js",
        lazyCompilation: {
          entries: true
        },
        context
      },
      {
        entry: "./esm/d.js",
        lazyCompilation: {
          entries: false,
          imports: true
        },
        context
      }
    ];
  },
  compiler(context, compiler) {
    new LazyCompilationTestPlugin().apply(compiler);
  },
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      compiler.watch({}, (err, multiStats) => {
        if (err) {
          compiler.close(() => {
            reject(err);
          });
          return;
        }

        const [statsA, statsB, statsC] = multiStats.stats;

        expect(
          statsA.toJson({ modules: true }).modules.every(module => {
            return !module.identifier.includes("lazy-compilation-proxy");
          })
        ).toBeTruthy();

        // second compiler lazy compile entry
        expect(
          statsB.toJson({ modules: true }).modules.find(module => {
            return (
              module.identifier.includes("lazy-compilation-proxy") &&
              module.identifier.replaceAll("\\", "/").includes("/esm/b.js")
            );
          })
        ).toBeDefined();

        // third compiler lazy compile dyn imports
        expect(
          statsC.toJson({ modules: true }).modules.find(module => {
            return (
              module.identifier.includes("lazy-compilation-proxy") &&
              module.identifier
                .replaceAll("\\", "/")
                .includes("/esm/d-dynamic.js")
            );
          })
        ).toBeDefined();

        compiler.close(err => {
          if (err) return reject(err);
          resolve();
        });
      });
    });
  }
}, {
  description: "should route colliding lazy compilation prefixes exactly",
  options(testContext) {
    return Array.from({ length: 11 }, (_, index) => {
      const name = `compiler-${index}`;
      return {
        context,
        mode: "development",
        name,
        target: "web",
        devtool: false,
        entry: "./esm/a.js",
        lazyCompilation: { entries: true, imports: false },
        output: {
          path: testContext.getDist(name),
          filename: "main.js",
          chunkFilename: "[name].js"
        }
      };
    });
  },
  async build(testContext, compiler) {
    const builds = [];
    const waiters = [];
    const nextBuild = () =>
      builds.length > 0
        ? Promise.resolve(builds.shift())
        : new Promise((resolve, reject) => {
          const timeout = setTimeout(
            () => reject(new Error("Timed out waiting for a lazy rebuild")),
            10000
          );
          waiters.push(value => {
            clearTimeout(timeout);
            resolve(value);
          });
        });
    const middleware = lazyCompilationMiddleware(compiler);

    compiler.watch({}, (error, stats) => {
      const value = {
        error:
          error ??
          (stats?.hasErrors()
            ? new Error(stats.toString({ all: false, errors: true }))
            : undefined),
        stats
      };
      (waiters.shift() ?? (build => builds.push(build)))(value);
    });

    try {
      const initial = await nextBuild();
      expect(initial.error).toBeUndefined();
      const bundle = initial.stats.stats[10].compilation
        .getAsset("main.js")
        .source.source()
        .toString();
      const encoded = bundle.match(/var data = ("(?:[^"\\]|\\.)*")/)?.[1];
      expect(encoded).toBeDefined();
      const moduleId = JSON.parse(encoded);

      await new Promise((resolve, reject) => {
        middleware(
          {
            body: [moduleId],
            method: "POST",
            url: "/_rspack/lazy/trigger__10?source=test"
          },
          {
            end: resolve,
            write() {},
            writeHead(status) {
              expect(status).toBe(200);
            }
          },
          reject
        ).catch(reject);
      });

      const updated = await nextBuild();
      expect(updated.error).toBeUndefined();
      expect(updated.stats.stats.map(child => child.compilation.name)).toEqual([
        "compiler-10"
      ]);
    } finally {
      await new Promise((resolve, reject) =>
        compiler.close(error => (error ? reject(error) : resolve()))
      );
    }
  }
}];
