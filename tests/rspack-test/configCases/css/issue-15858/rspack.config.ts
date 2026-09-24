import { defineConfig } from '@rspack/cli';

export default defineConfig(
  ['single', 'multiple', 'single-preloaded', 'multiple-preloaded'].map(
    (name) => ({
      name,
      mode: 'development',
      target: 'web',
      devtool: false,
      entry: {
        main: './index.js',
        other: './feature.js',
      },
      output: {
        filename: `[name].${name}.js`,
        chunkFilename: `[name].${name}.js`,
        cssFilename: `[name].${name}.css`,
        cssChunkFilename: `[name].${name}.css`,
        publicPath: 'https://test.cases/path/',
      },
      module: {
        rules: [
          {
            test: /\.css$/,
            type: 'css',
            parser: { exportType: 'link' },
          },
        ],
      },
      experiments: { css: true },
      optimization: {
        runtimeChunk: name.startsWith('single') ? 'single' : 'multiple',
        chunkIds: 'named',
        minimize: false,
        splitChunks: {
          cacheGroups: {
            default: false,
            defaultVendors: false,
            styles: {
              test: /\.css$/,
              chunks: 'all',
              name: 'shared-css',
              enforce: true,
            },
          },
        },
      },
    }),
  ),
);
