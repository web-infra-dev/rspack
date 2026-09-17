import { rspack } from '@rspack/core';

const { RawSource, ConcatSource } = rspack.sources;

/** @type {import("@rspack/core").Configuration} */
export default {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  node: {
    __dirname: false,
    __filename: false,
  },
  entry: {
    main: './index.js',
  },
  output: {
    publicPath: 'auto',
    cssFilename: 'css/[name].css',
  },
  resolve: {
    alias: {
      '@': import.meta.dirname,
    },
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
        generator: {
          filename: 'image/[name][ext]',
        },
      },
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  plugins: [],
};
