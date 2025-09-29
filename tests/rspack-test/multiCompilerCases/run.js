/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig[]} */
module.exports = [(() => {
  let called = 0;
  return {
    description: "should trigger 'run' for each child compiler",
    async compiler(context, compiler) {
      compiler.hooks.run.tap("MultiCompiler test", () => called++);
    },
    async build(context, compiler) {
      return new Promise((resolve, reject) => {
        compiler.run(err => {
          if (err) {
            return reject(err);
          }
          expect(called).toBe(2);
          compiler.close(resolve);
        });
      });
    },
  }
})(), (() => {
  let called = 0;
  return {
    description: "should trigger 'watchRun' for each child compiler",
    async compiler(context, compiler) {
      compiler.hooks.watchRun.tap("MultiCompiler test", () => called++);
    },
    async build(context, compiler) {
      return new Promise((resolve, reject) => {
        compiler.watch({ aggregateTimeout: 1000 }, err => {
          if (err) {
            return reject(err);
          }
          expect(called).toBe(2);
          compiler.close(resolve);
        });
      });
    },
  };
})()];
