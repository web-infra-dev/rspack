import { defineConfig, definePlugin } from '@rspack/cli';
import { type Configuration, WebpackError } from '@rspack/core';

export default defineConfig(
  [0, 1, 10, 2, 20, 11, 12, 13, 3, 30].map<Configuration>((n) => ({
    name: `${n % 10} errors ${(n / 10) | 0} warnings`,
    mode: 'development',
    output: {
      filename: `${n}.js`,
    },
    entry: './index.js',
    plugins: [
      definePlugin((compiler) => {
        compiler.hooks.compilation.tap('Test', (compilation) => {
          const err = Object.assign(new WebpackError('Test'), {
            details: 'Error details',
          });
          for (let i = n % 10; i > 0; i--) compilation.errors.push(err);
          for (let i = (n / 10) | 0; i > 0; i--) compilation.warnings.push(err);
        });
      }),
    ],
    stats: {
      assets: true,
      modules: true,
    },
  })),
);
