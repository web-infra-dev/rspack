import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';

class Plugin {
  apply(compiler: Compiler) {
    let testRuned = false;
    compiler.hooks.assetEmitted.tap('test', (filename, info) => {
      const { targetPath, content } = info;
      if (targetPath.endsWith('.css')) {
        testRuned = true;
        expect(content).toBeDefined();
        expect(filename.endsWith('.css?v=2')).toBeTruthy();
      }
    });
    compiler.hooks.done.tap('test', () => {
      expect(testRuned).toBeTruthy();
    });
  }
}

export default defineConfig({
  target: 'web',
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'asset/resource',
      },
    ],
  },
  plugins: [new Plugin()],
});
