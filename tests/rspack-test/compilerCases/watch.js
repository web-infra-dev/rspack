let counterBeforeCompile = 0;
let counterDone = 0;
let counterHandler = 0;
let calls = 0;

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = [{
  description: "should only compile a single time",
  options(context) {
    return {
      context: context.getSource("watch"),
      watch: true,
      mode: "development",
      experiments: {
        futureDefaults: true
      },
      entry: "./src/index.js",
      module: {
        rules: [
          {
            test: /\.js$/,
            use: "some-loader"
          }
        ]
      },
      plugins: [
        c => {
          c.hooks.beforeCompile.tap("test", () => {
            counterBeforeCompile++;
          });
          c.hooks.done.tap("test", () => {
            counterDone++;
          });
        }
      ]
    };
  },
  compilerCallback(err, stats) {
    if (err) throw err;
    if (stats.hasErrors()) throw new Error(stats.toString());
    counterHandler++;
  },
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      compiler.hooks.afterDone.tap("afterDoneRunTest", () => {
        compiler.close(resolve);
      });
    });
  },
  async check() {
    expect(counterBeforeCompile).toBe(1);
    expect(counterDone).toBe(1);
    expect(counterHandler).toBe(1);
  }
}, {
  description: "should correctly emit asset when invalidation occurs again",
  options(context) {
    return {
      mode: "development",
      context: context.getSource("watch"),
      entry: "./src/index.js",
    };
  },
  async compiler(context, compiler) {
    compiler.hooks.emit.tap("Test", () => {
      calls++;
    });
    // Ensure the second invalidation can occur during compiler running
    let once = false;
    compiler.hooks.afterCompile.tapAsync("LongTask", (_, cb) => {
      if (once) return cb();
      once = true;
      setTimeout(() => {
        cb();
      }, 1000);
    });
  },
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      compiler.watch({}, (err, stats) => {
        if (err) return reject(err);
      });
      compiler.watching.invalidate();
      setTimeout(() => {
        compiler.watching.invalidate();
      }, 50);
      compiler.hooks.done.tap("Test", () => {
        compiler.close(resolve);
      });
    });
  },
  async check() {
    expect(calls).toBe(2);
  }
}];
