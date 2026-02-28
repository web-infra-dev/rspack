/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig[]} */
module.exports = [{
  description: "should run again correctly after first compilation",
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      compiler.run((err, stats) => {
        if (err) return reject(err);

        compiler.run((err, stats) => {
          if (err) return reject(err);
          compiler.close(resolve);
        });
      });
    });
  },
}, {
  description: "should watch again correctly after first compilation",
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      compiler.run((err, stats) => {
        if (err) return reject(err);

        compiler.watch({}, (err, stats) => {
          if (err) return reject(err);
          compiler.close(resolve);
        });
      });
    });
  },
}, {
  description: "should run again correctly after first closed watch",
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      const watching = compiler.watch({}, (err, stats) => {
        if (err) return reject(err);
      });
      watching.close(() => {
        compiler.run((err, stats) => {
          if (err) return reject(err);
          compiler.close(resolve);
        });
      });
    });
  },
}, {
  description: "should watch again correctly after first closed watch",
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      const watching = compiler.watch({}, (err, stats) => {
        if (err) return reject(err);
      });
      watching.close(() => {
        compiler.watch({}, (err, stats) => {
          if (err) return reject(err);
          compiler.close(resolve);
        });
      });
    });
  },
}];
