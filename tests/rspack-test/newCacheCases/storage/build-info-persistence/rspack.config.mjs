import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';

const customFieldValues = [];

export default defineConfig({
  entry: './d.js',
  cache: { type: 'persistent' },
  module: {
    rules: [
      {
        test: /\.js$/,
        use: [
          {
            loader: path.join(import.meta.dirname, 'build-info-loader.mjs'),
          },
        ],
      },
    ],
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap('PersistBuildInfo', (compilation) => {
          compilation.hooks.finishModules.tap('PersistBuildInfo', (modules) => {
            for (const module of modules) {
              customFieldValues.push(module.buildInfo.foo);
            }
          });
        });
        compiler.hooks.done.tap('PersistBuildInfo', () => {
          expect(customFieldValues.length).toBeGreaterThan(0);
          customFieldValues.forEach((foo) => {
            expect(foo).toBeTruthy();
          });
        });
      },
    }),
  ],
});
