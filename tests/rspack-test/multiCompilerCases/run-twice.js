const { createFsFromVolume, Volume } = require("memfs");

/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig[]} */
module.exports = [{
  description: "should not be running twice at a time (run)",
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      compiler.run((err, stats) => {
        if (err) return reject(err);
      });
      compiler.run((err, stats) => {
        if (err) {
          compiler.close(resolve);
        }
      });
    });
  },
}, {
  description: "should not be running twice at a time (watch)",
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      compiler.watch({}, (err, stats) => {
        if (err) return reject(err);
      });
      compiler.watch({}, (err, stats) => {
        if (err) {
          compiler.close(resolve);
        }
      });
    });
  },
}, {
  description: "should not be running twice at a time (run - watch)",
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      compiler.run((err, stats) => {
        if (err) return reject(err);
      });
      compiler.watch({}, (err, stats) => {
        if (err) {
          compiler.close(resolve);
        }
      });
    });
  },
}, {
  description: "should not be running twice at a time (watch - run)",
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      compiler.watch({}, (err, stats) => {
        if (err) return reject(err);
      });
      compiler.run((err, stats) => {
        if (err) {
          compiler.close(resolve);
        }
      });
    });
  },
}, {
  description: "should not be running twice at a time (instance cb)",
  options(context) {
    return {
      context: __dirname,
      mode: "production",
      entry: "./c",
      output: {
        path: "/",
        filename: "bundle.js"
      }
    };
  },
  compilerCallback(err, stats) {
  },
  compiler(context, compiler) {
    compiler.outputFileSystem = createFsFromVolume(new Volume());
  },
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      compiler.run((err, stats) => {
        if (err) {
          compiler.close(resolve);
        }
      });
    });
  },
}];
