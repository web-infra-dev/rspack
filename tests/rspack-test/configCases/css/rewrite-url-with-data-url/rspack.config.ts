import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  node: false,
  output: {
    cssFilename: 'css/[name].css',
  },

  module: {
    generator: {
      'css/auto': {
        exportsOnly: false,
      },
    },
    rules: [
      {
        test: /\.png$/i,
        type: 'asset',
        parser: {
          dataUrlCondition: {
            maxSize: 30000,
          },
        },
        generator: {
          filename: 'image/[name].[contenthash:8][ext]',
        },
      },
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
});
