const { RuntimeGlobals } = require("@rspack/core");

let called = 0;

class Plugin {
  apply(compiler) {
    compiler.hooks.thisCompilation.tap("TestFakePlugin", compilation => {
      compilation.hooks.runtimeRequirementInTree.tap("TestFakePlugin", (chunk, set) => {
        expect(chunk.name).toBe("main");
        if (called === 0) {
          expect(set.has(RuntimeGlobals.chunkName)).toBeFalsy();
          set.add(RuntimeGlobals.chunkName);
          set.add(RuntimeGlobals.ensureChunk);
          set.add(RuntimeGlobals.ensureChunkHandlers);
        } else if (called === 1) {
          expect(set.has(RuntimeGlobals.chunkName)).toBeTruthy();
          expect(set.has(RuntimeGlobals.ensureChunk)).toBeTruthy();
          expect(set.has(RuntimeGlobals.ensureChunkHandlers)).toBeTruthy();
          expect(set.has(RuntimeGlobals.hasOwnProperty)).toBeFalsy();
        } else if (called === 2) {
          expect(set.has(RuntimeGlobals.hasOwnProperty)).toBeTruthy();
        } else {
          throw new Error("should not call more than 3 times");
        }
        called++;
      });
    });
  }
}
/**@type {import("@rspack/core").Configuration}*/
module.exports = {
  context: __dirname,
  plugins: [new Plugin()]
};
