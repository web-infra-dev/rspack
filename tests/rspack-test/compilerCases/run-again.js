const { createFsFromVolume, Volume } = require("memfs");

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [{
  description: "should run again correctly after first compilation",
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
      compiler.run((err, stats1) => {
        if (err) return reject(err);

        compiler.run((err, stats2) => {
          if (err) return reject(err);
          expect(stats1.toString({ all: true })).toBeTypeOf("string");
          resolve();
        });
      });
    });
  },
}, {
  skip: true,
  description: "should run again correctly after first closed watch",
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
      });
      watching.close(() => {
        compiler.run((err, stats) => {
          if (err) return reject(err);
          resolve();
        });
      });
    });
  },
}, {
  description: "should run again correctly inside afterDone hook",
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
      let once = true;
      compiler.hooks.afterDone.tap("RunAgainTest", () => {
        if (!once) return;
        once = false;
        compiler.run((err, stats) => {
          if (err) return reject(err);
          resolve();
        });
      });
      compiler.run((err, stats) => {
        if (err) return reject(err);
      });
    });
  },
}, {
  description: "should watch again correctly after first compilation",
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
      compiler.run((err, stats) => {
        if (err) return reject(err);

        const watching = compiler.watch({}, (err, stats) => {
          if (err) return reject(err);
          watching.close(resolve);
        });
      });
    });
  },
}, {
  skip: true,
  description: "should run again correctly after first closed watch",
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
      });
      watching.close(() => {
        compiler.watch({}, (err, stats) => {
          if (err) return reject(err);
          resolve();
        });
      });
    });
  },
}];
