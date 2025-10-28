const { createFsFromVolume, Volume } = require("memfs");
const { Stats } = require("@rspack/core");

let watchStats = null;

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [{
  description: "should not emit on errors",
  error: true,
  options(context) {
    return {
      entry: "./missing",
      output: {
        filename: "bundle.js"
      },
    };
  },
  async compiler(context, compiler) {
    compiler.outputFileSystem = createFsFromVolume(new Volume());
  },
  async check({ compiler }) {
    expect(compiler.outputFileSystem.existsSync("/bundle.js")).toBe(false);
  }
}, {
  description: "should not emit on errors (watch)",
  error: true,
  options(context) {
    return {
      entry: "./missing",
      output: {
        filename: "bundle.js"
      },
    };
  },
  async compiler(context, compiler) {
    compiler.outputFileSystem = createFsFromVolume(new Volume());
  },
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      const watching = compiler.watch({}, (err, stats) => {
        watching.close(() => {
          if (err) return reject(err);
          resolve();
        });
      });
    });
  },
  async check({ compiler }) {
    expect(compiler.outputFileSystem.existsSync("/bundle.js")).toBe(false);
  }
}, {
  description: "should not emit compilation errors in async (watch)",
  error: true,
  options(context) {
    return {
      entry: "./missing-file",
      output: {
        filename: "bundle.js"
      },
    };
  },
  async compiler(context, compiler) {
    compiler.outputFileSystem = createFsFromVolume(new Volume());
  },
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      const watching = compiler.watch({}, (err, stats) => {
        watching.close(() => {
          if (err) return reject(err);
          watchStats = stats;
          resolve();
        });
      })
    });
  },
  async check() {
    expect(watchStats).toBeInstanceOf(Stats);
  }
}];
