const { basename, defineCompileCase } = require("@rspack/test-tools");
class MyPlugin {
  apply(compiler) {
    compiler.hooks.compilation.tap("Plugin", compilation => {
      compilation.hooks.finishModules.tap("Plugin", _ => {
        const stats = compilation.getStats().toJson({
          all: false,
          modules: true,
        });
        const modules = stats.modules;

        expect(modules).toBeDefined();
      });
    });
  }
}


defineCompileCase(Utils.basename(__filename), {
  description: "should not panic get stats when chunkGraphModule is not available",
  options(context) {
    return {
      context: context.getSource(),
      entry: "./d",
      plugins: [new MyPlugin()]
    };
  }
});