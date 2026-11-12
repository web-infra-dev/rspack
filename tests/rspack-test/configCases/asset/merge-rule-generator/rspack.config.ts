import type { Configuration } from '@rspack/core';

const generatorRule = {
  test: /\.json$/,
  parser: {
    wrappedContextRegExp: { source: '[', flags: '' },
  },
  generator: Object.defineProperty(
    { dataUrl: { encoding: false, mimetype: 'application/x-rspack-dom' } },
    'filename',
    { get: () => 'data/[name].[contenthash][ext]' },
  ),
};
const typeRule = {
  with: { type: 'download' },
  type: 'asset/resource',
};
const inlineTypeRule = {
  with: { type: 'inline' },
  type: 'asset/inline',
};

export default [
  [generatorRule, typeRule, inlineTypeRule],
  [typeRule, inlineTypeRule, generatorRule],
].map((rules): Configuration => ({
  output: {
    assetModuleFilename: '[contenthash][ext]',
    publicPath: '',
  },
  module: { rules },
}));
