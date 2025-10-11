

let watcher = null;

function close(watcher, callback) {
  return new Promise(res => {
    const onClose = () => {
      callback();
      res();
    };
    watcher.close(onClose);
  });
}
/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
  description: "multiple calls watcher: each callback should be called",
  options(context) {
    return {
      mode: "development",
      entry: "./a.js",
      output: {
        filename: "bundle.js"
      }
    };
  },
  async compiler(context, compiler) {

  },
  async build(context, compiler) {
    watcher = compiler.watch({ poll: 300 }, () => { });

  },
  async check() {
    let num = 0;

    await Promise.all([
      close(watcher, () => (num += 1)),
      close(watcher, () => (num += 10))
    ]);
    await Promise.all([
      close(watcher, () => (num += 100)),
      close(watcher, () => (num += 1000))
    ]);

    expect(num).toBe(1111);
  }
}
