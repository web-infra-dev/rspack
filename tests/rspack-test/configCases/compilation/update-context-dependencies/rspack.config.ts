import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';
import path from 'node:path';

const PLUGIN_NAME = 'plugin';
const TEST_DIR = path.resolve(import.meta.dirname, './src/');

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.afterCompile.tap(PLUGIN_NAME, (compilation) => {
      compilation.contextDependencies.add(TEST_DIR);
      expect(compilation.contextDependencies.has(TEST_DIR)).toBeTruthy();
      expect(
        [...compilation.contextDependencies].includes(TEST_DIR),
      ).toBeTruthy();
    });
  }
}

export default defineConfig({
  entry: './index.js',
  plugins: [new Plugin()],
});
