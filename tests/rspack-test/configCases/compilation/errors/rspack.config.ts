import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';
import { fileURLToPath } from 'node:url';

const PLUGIN_NAME = 'plugin';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.make.tap(PLUGIN_NAME, (compilation) => {
      compilation.hooks.processAssets.tap(PLUGIN_NAME, () => {
        expect(compilation.errors.length).toBe(2);
        expect(compilation.errors[0].message).toMatch(/emitted error1/);
        expect(compilation.errors[1].message).toMatch(/emitted error2/);

        expect(compilation.warnings.length).toBe(2);
        expect(compilation.warnings[0].message).toMatch(/emitted warning1/);
        expect(compilation.warnings[1].message).toMatch(/emitted warning2/);

        compilation.errors = [];
        expect(compilation.errors.length).toBe(0);

        expect(compilation.warnings.length).toBe(2);
        expect(compilation.warnings[0].message).toMatch(/emitted warning1/);
        expect(compilation.warnings[1].message).toMatch(/emitted warning2/);

        compilation.warnings = [];
        expect(compilation.warnings.length).toBe(0);
      });
    });
  }
}

export default defineConfig({
  entry: './index.js',
  plugins: [new Plugin()],
  module: {
    rules: [
      {
        test: /\.js$/,
        use: [
          {
            loader: fileURLToPath(import.meta.resolve('./loader.mjs')),
          },
        ],
      },
    ],
  },
});
