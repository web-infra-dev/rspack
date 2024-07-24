const { RuntimeGlobals } = require("@rspack/core");

class Plugin {
  apply(compiler) {
    compiler.hooks.thisCompilation.tap("TestFakePlugin", compilation => {
      compilation.hooks.additionalTreeRuntimeRequirements.tap("TestFakePlugin", (_, set) => {
        expect(set.has(RuntimeGlobals.chunkName)).toBeFalsy();
        expect(set.has(RuntimeGlobals.getFullHash)).toBeTruthy();
        set.add(RuntimeGlobals.chunkName);
        set.delete(RuntimeGlobals.getFullHash);
      });
    });
  }
}
/**@type {import("@rspack/core").Configuration}*/
module.exports = {
  context: __dirname,
  plugins: [new Plugin()]
};
