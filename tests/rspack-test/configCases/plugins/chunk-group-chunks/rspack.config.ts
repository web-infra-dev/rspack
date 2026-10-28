import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

function plugin(compiler: Compiler) {
  compiler.hooks.compilation.tap('plugin', (compilation) => {
    compilation.hooks.processAssets.tap('plugin', () => {
      const entryA = compilation.entrypoints.get('a');
      expect(entryA?.chunks.map((c) => c.name)).toEqual(['a']);
      const entryB = compilation.entrypoints.get('b');
      expect(entryB?.chunks.map((c) => c.name)).toEqual(['b']);
    });
  });
}

export default defineConfig({
  entry: {
    a: './entry1.js',
    b: './entry2.js',
  },
  output: {
    filename: '[name].js',
  },
  plugins: [plugin],
});
