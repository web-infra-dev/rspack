class Plugin {
  apply(compiler) {
    const RuntimeGlobals = compiler.rspack.RuntimeGlobals;
    compiler.hooks.thisCompilation.tap('TestFakePlugin', (compilation) => {
      compilation.hooks.additionalTreeRuntimeRequirements.tap(
        'TestFakePlugin',
        (_, set) => {
          if (!globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
            expect(set.has(RuntimeGlobals.chunkName)).toBeFalsy();
          }
          expect(set.has(RuntimeGlobals.getFullHash)).toBeTruthy();
          set.add(RuntimeGlobals.chunkName);
          set.delete(RuntimeGlobals.getFullHash);
        },
      );
    });
  }
}
/**@type {import("@rspack/core").Configuration}*/
export default {
  context: import.meta.dirname,
  plugins: [new Plugin()],
};
