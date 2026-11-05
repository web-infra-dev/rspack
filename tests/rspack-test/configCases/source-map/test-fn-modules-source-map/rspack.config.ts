import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  node: {
    __dirname: false,
    __filename: false,
  },
  entry: {
    bundle0: ['./index.js'],
    bundle1: ['./test1.js', './test2.js'],
  },
  output: {
    filename: '[name].js',
  },
  plugins: [
    new rspack.EvalSourceMapDevToolPlugin({
      // @ts-expect-error Rspack does not support a function test for SourceMapDevToolPlugin yet.
      test: (str: string) => {
        if (str.endsWith('.js')) return true;
        return false;
      },
      exclude: /test2\.js/,
      module: true,
      columns: false,
    }),
  ],
});
