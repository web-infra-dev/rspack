import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';

const PLUGIN_NAME = 'plugin';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
      compilation.hooks.finishModules.tap(PLUGIN_NAME, () => {
        expect(Array.from(compilation.entrypoints.keys())).toEqual([]);
      });

      compilation.hooks.afterProcessAssets.tap(PLUGIN_NAME, () => {
        expect(Array.from(compilation.entrypoints.keys())).toEqual([
          'main',
          'foo',
          'bar',
        ]);
      });
    });
  }
}

export default defineConfig({
  entry: {
    main: {
      import: './index.js',
    },
    foo: {
      import: './foo.js',
      asyncChunks: true,
    },
    bar: {
      import: './bar.js',
      asyncChunks: true,
    },
  },
  plugins: [new Plugin()],
});
