import path from 'node:path';
const base = {
  mode: 'production',
  entry: './src/index.js',
  devtool: false,
  output: {
    filename: 'main.js',
    assetModuleFilename: '[contenthash][ext]',
  },
  module: {
    rules: [
      {
        test: /\.(png|jpg)$/,
        type: 'asset/resource',
      },
    ],
  },
  stats: 'normal',
  context: import.meta.dirname,
};

/** @type {import("@rspack/core").Configuration[]} */
export default [
  {
    ...base,
    output: {
      ...base.output,
      path: path.resolve(import.meta.dirname, './dist/enable'),
    },
    optimization: {
      realContentHash: true,
    },
  },
  {
    ...base,
    output: {
      ...base.output,
      path: path.resolve(import.meta.dirname, './dist/disable'),
    },
    optimization: {
      realContentHash: false,
    },
  },
];
