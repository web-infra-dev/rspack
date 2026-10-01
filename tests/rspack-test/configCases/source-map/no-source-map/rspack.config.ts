import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

const plugins = [
  (compiler: Compiler) => {
    compiler.hooks.emit.tap('Test', (compilation) => {
      for (const asset of compilation.getAssets()) {
        const result = asset.source.sourceAndMap();
        try {
          expect(result.map).toBe(null);
        } catch (e) {
          if (e instanceof Error) {
            e.message += `\nfor asset ${asset.name}`;
          }
          throw e;
        }
      }
    });
  },
];

export default defineConfig([
  {
    mode: 'development',
    devtool: false,
    plugins,
  },
  {
    mode: 'production',
    devtool: false,
    plugins,
  },
  {
    mode: 'production',
    devtool: false,
    optimization: {
      minimize: true,
    },
    plugins,
  },
]);
