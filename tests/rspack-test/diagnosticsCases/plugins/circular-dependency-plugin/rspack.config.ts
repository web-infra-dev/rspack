import { defineConfig, definePlugin } from '@rspack/cli';
import { CircularDependencyRspackPlugin } from '@rspack/core';

const startFn = rstest.fn();
const endFn = rstest.fn();

export default defineConfig({
  entry: {
    aa: './require-circular/d.js',
    bb: './import-circular/index.js',
    cc: './no-cycle/index.js',
    dd: './ignore-circular/a.js',
    ee: './multiple-circular/a.js',
    ff: {
      import: './multiple-circular/a.js',
      layer: 'f',
    },
    gg: './dynamic-circular/index.js',
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        loader: './loader.mjs',
      },
    ],
  },
  plugins: [
    new CircularDependencyRspackPlugin({
      failOnError: false,
      exclude: /(ignore-circular|loader)/,
      onStart(_compilation) {
        expect(typeof _compilation.errors === 'object').toBeTruthy();
        expect(typeof _compilation.errors.push === 'function').toBeTruthy();
        startFn();
      },
      onEnd(_compilation) {
        endFn();
      },
    }),
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('done', () => {
          expect(startFn).toHaveBeenCalled();
          expect(endFn).toHaveBeenCalled();
        });
      },
    }),
  ],
});
