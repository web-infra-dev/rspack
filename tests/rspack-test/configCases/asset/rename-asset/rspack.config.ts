import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap('Test', (compilation) => {
      compilation.hooks.processAssets.tap(
        {
          name: 'Test',
          stage: -100,
        },
        () => {
          compilation.renameAsset('chunk.js', 'renamed.js');
        },
      );
    });
  }
}

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  context: import.meta.dirname,
  output: {
    chunkFilename: 'chunk.js',
  },
  plugins: [new Plugin()],
});
