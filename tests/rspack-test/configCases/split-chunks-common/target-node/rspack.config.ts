import assert from 'node:assert/strict';
import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    name: 'default',
    entry: './index',
    target: 'node',
    output: {
      filename: 'default-[name].js',
      library: { type: 'commonjs2' },
    },
    optimization: {
      splitChunks: {
        minSize: 1,
        chunks: 'all',
      },
    },
  },
  {
    name: 'many-vendors',
    entry: './index',
    target: 'node',
    output: {
      filename: 'many-vendors-[name].js',
      library: { type: 'commonjs2' },
    },
    optimization: {
      splitChunks: {
        minSize: 1,
        chunks: 'all',
        maxInitialRequests: Infinity,
        cacheGroups: {
          default: false,
          defaultVendors: false,
          vendors: {
            test: /node_modules/,
            name: (m) => {
              const name = m.nameForCondition();
              assert(name);
              const match = name.match(/([b-d]+)\.js$/);
              if (match) return 'vendors-' + match[1];
            },
          },
        },
      },
    },
  },
]);
