import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

const common = defineConfig({
  entry: {
    main: './index.js',
    test: './test.js',
    mixed: './mixed.js',
    normal: './normal.js',
  },
  optimization: {
    splitChunks: {
      cacheGroups: {
        test: {
          chunks: 'all',
          minSize: 0,
          test: /test\.js/,
          name: 'test',
        },
      },
    },
  },
  module: {
    rules: [
      {
        test: /(?:mixed|normal)-asset\.js$/,
        type: 'asset/resource',
      },
      {
        test: /normal\.js$/,
        parser: {
          url: true,
        },
      },
    ],
    parser: {
      javascript: {
        url: 'new-url-relative',
        importMeta: false,
      },
    },
    generator: {
      asset: {
        filename: 'asset/static-[name].js',
      },
    },
  },
});

export default defineConfig([
  {
    ...common,
    optimization: {
      ...common.optimization,
      concatenateModules: true,
    },
    output: {
      module: true,
      filename: `[name]-0.mjs`,
    },
    plugins: [
      new rspack.DefinePlugin({
        INDEX: 0,
      }),
    ],
  },
  {
    ...common,
    optimization: {
      ...common.optimization,
      concatenateModules: false,
    },
    output: {
      module: true,
      filename: `[name]-1.mjs`,
    },
    plugins: [
      new rspack.DefinePlugin({
        INDEX: 1,
      }),
    ],
  },
]);
