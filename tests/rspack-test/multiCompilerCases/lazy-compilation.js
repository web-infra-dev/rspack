const { LazyCompilationTestPlugin } = require("@rspack/test-tools");
const path = require("path");
const context = path.join(__dirname, "../fixtures");

/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig} */
module.exports = {
  description: "compiler has unique lazy compilation config",
  options() {
    return [
      {
        entry: "./esm/a.js",
        context
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
          statsA.toJson().modules.every(module => {
            return !module.identifier.includes("lazy-compilation-proxy");
          })
        ).toBeTruthy();

        // second compiler lazy compile entry
        expect(
          statsB.toJson().modules.find(module => {
            return (
              module.identifier.includes("lazy-compilation-proxy") &&
              module.identifier.replaceAll("\\", "/").includes("/esm/b.js")
            );
          })
        ).toBeDefined();

        // third compiler lazy compile dyn imports
        expect(
          statsC.toJson().modules.find(module => {
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
};