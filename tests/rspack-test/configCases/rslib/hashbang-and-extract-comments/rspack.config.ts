import { defineConfig } from '@rspack/cli';
import { type Configuration, rspack as rspack2 } from '@rspack/core';

const {
  rspack,
  experiments: { RslibPlugin },
} = rspack2;

const baseConfig = (i: number, mjs = false): Configuration => ({
  entry: {
    index: {
      import: './index.js',
      filename: `bundle${i}${mjs ? '.mjs' : '.js'}`,
    },
  },
  target: 'node',
  node: mjs
    ? {}
    : {
        __filename: false,
        __dirname: false,
      },
  optimization: {
    minimize: true,
    minimizer: [
      new rspack.SwcJsMinimizerRspackPlugin({
        extractComments: true,
      }),
    ],
  },
});

export default defineConfig([
  // CJS output
  {
    ...baseConfig(0),
    output: {
      library: {
        type: 'commonjs',
      },
    },
    plugins: [new RslibPlugin()],
  },
  // ESM output
  {
    ...baseConfig(1, true),
    externals: {
      os: 'module os',
    },
    output: {
      module: true,
      library: {
        type: 'modern-module',
      },
    },
    plugins: [new RslibPlugin()],
  },
  // Test entry
  {
    entry: './test.js',
    target: 'node',
    output: {
      module: true,
    },
  },
]);
