let called = 0;
/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig} */
module.exports = [{
  description: "should emit 'watch-close' when using multi-compiler mode and the compiler is not running",
  options(context) {
    return [{
      entry: "./a.js"
    }];
  },
  async build(context, compiler) {
    return new Promise((resolve, reject) => {
      const watcher = compiler.watch({}, (err, stats) => {
        if (err) return reject(err);
        resolve();
      });

      compiler.hooks.watchClose.tap("WatcherEventsTest", () => {
        called = true;
      });

      compiler.hooks.done.tap("WatcherEventsTest", () => {
        watcher.close();
      });
    });
  },
  async check() {
    expect(called).toBe(true);
  }
}];
