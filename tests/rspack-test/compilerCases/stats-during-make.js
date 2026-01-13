class StatsDuringMakePlugin {
  apply(compiler) {
    compiler.hooks.compilation.tap("StatsDuringMakePlugin", compilation => {
      let called = false;
      compilation.hooks.buildModule.tap("StatsDuringMakePlugin", () => {
        if (called) return;
        called = true;
        const stats = compilation.getStats().toJson({
          all: false,
          modules: true,
        });
        expect(stats.modules).toBeDefined();
      });
    });
  }
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
  description:
    "should not panic when accessing stats during make module graph",
  options(context) {
    return {
      context: context.getSource(),
      entry: "./d",
      plugins: [new StatsDuringMakePlugin()],
    };
  },
};
