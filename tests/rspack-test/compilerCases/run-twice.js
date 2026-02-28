const { createFsFromVolume, Volume } = require("memfs");

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [{
  description: "should not be running twice at a time (run)",
  options(context) {
    return {
      entry: "./c",
    };
  },
  error: true,
  async compiler(context, compiler) {
    compiler.outputFileSystem = createFsFromVolume(new Volume());
  },
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      compiler.run((err, stats) => {
        if (err) return reject(err);
      });
      compiler.run((err, stats) => {
        if (err) return resolve();
      });
    });
  },
}, {
  description: "should not be running twice at a time (run)",
  options(context) {
    return {
      entry: "./c",
    };
  },
  error: true,
  async compiler(context, compiler) {
    compiler.outputFileSystem = createFsFromVolume(new Volume());
  },
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      compiler.watch({}, (err, stats) => {
        if (err) return reject(err);
      });
      compiler.watch({}, (err, stats) => {
        if (err) return resolve();
      });
    });
  },
}, {
  description: "should not be running twice at a time (run - watch)",
  options(context) {
    return {
      entry: "./c",
    };
  },
  error: true,
  async compiler(context, compiler) {
    compiler.outputFileSystem = createFsFromVolume(new Volume());
  },
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      compiler.watch({}, (err, stats) => {
        if (err) return reject(err);
      });
      compiler.run((err, stats) => {
        if (err) return resolve();
      });
    });
  },
}, {
  description: "should not be running twice at a time (run - watch)",
  options(context) {
    return {
      entry: "./c",
    };
  },
  error: true,
  async compiler(context, compiler) {
    compiler.outputFileSystem = createFsFromVolume(new Volume());
  },
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      compiler.run((err, stats) => {
        if (err) return reject(err);
      });
      compiler.watch({}, (err, stats) => {
        if (err) return resolve();
      });
    });
  },
}, {
  description: "should not be running twice at a time (instance cb)",
  options(context) {
    return {
      entry: "./c",
    };
  },
  compilerCallback(error, stats) {
  },
  error: true,
  async compiler(context, compiler) {
    compiler.outputFileSystem = createFsFromVolume(new Volume());
  },
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      compiler.run((err, stats) => {
        if (err) return resolve();
      })
    });
  },
}];
