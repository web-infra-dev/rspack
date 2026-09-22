import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap('Test', (compilation) => {
      compilation.hooks.processAssets.tap('Test', () => {
        const module = Array.from(compilation.modules)[0];
        const moduleHash = compilation.chunkGraph.getModuleHash(module, 'main');
        expect(moduleHash).toBeTruthy();
      });
    });
  }
}

export default defineConfig({
  target: 'web',
  node: false,
  entry: {
    main: './index.js',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    sideEffects: false,
  },
  plugins: [new Plugin()],
});
