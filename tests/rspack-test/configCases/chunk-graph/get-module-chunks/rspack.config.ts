import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap('Test', (compilation) => {
      compilation.hooks.processAssets.tap('Test', () => {
        const chunk = Array.from(compilation.chunks)[0];
        const modules = compilation.chunkGraph.getChunkModules(chunk);
        expect(modules[0]).toHaveProperty(
          'resource',
          path.join(import.meta.dirname, 'index.js'),
        );
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
