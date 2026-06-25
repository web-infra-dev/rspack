const { RawSource, ConcatSource } = require('webpack-sources');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
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
      '@': __dirname,
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
