import type { Configuration } from '@rspack/core';

const generatorRule = {
  test: /\.json$/,
  generator: { filename: 'data/[name].[contenthash][ext]' },
};
const typeRule = {
  with: { type: 'download' },
  type: 'asset/resource',
};

export default [
  [generatorRule, typeRule],
  [typeRule, generatorRule],
].map((rules): Configuration => ({
  output: {
    assetModuleFilename: '[contenthash][ext]',
    publicPath: '',
  },
  module: { rules },
}));
