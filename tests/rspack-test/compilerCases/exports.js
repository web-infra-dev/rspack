const { RuntimeGlobals } = require("@rspack/core");

class MyPlugin {
  apply(compiler) {
    expect(typeof compiler.rspack).toBe("function");
    expect(compiler.rspack.sources).toBeTruthy();
    expect(compiler.rspack.Compilation).toBeTruthy();
    expect(compiler.rspack.RuntimeGlobals).toBeTruthy();
    expect(compiler.rspack.RuntimeGlobals).not.toBe(RuntimeGlobals);
  }
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
  description: "should be called every compilation",
  options(context) {
    return {
      context: context.getSource(),
      entry: "./d",
      plugins: [new MyPlugin()]
    };
  },
  async build(_, compiler) {
    await new Promise(resolve => {
      compiler.run(() => {
        resolve();
      });
    });
  },
};
