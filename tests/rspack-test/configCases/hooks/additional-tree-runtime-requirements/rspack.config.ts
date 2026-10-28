import { defineConfig } from '@rspack/cli';
import { type Compiler } from '@rspack/core';

class Plugin {
  apply(compiler: Compiler) {
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
export default defineConfig({
  context: import.meta.dirname,
  plugins: [new Plugin()],
});
