import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  entry: {
    bundle0: './index.js',
    'bundle-import': './import.js',
    'bundle-amd-require': './amd-define.js',
    'bundle-amd-define': './amd-require.js',
    'bundle-commonjs': './commonjs.js',
    'bundle-require.resolve': './require.resolve.js',
  },
  output: {
    filename: '[name].js',
  },
  module: {
    // @ts-expect-error Preserve the unsupported webpack context options in this fixture.
    exprContextCritical: false,
  },
  node: {
    __dirname: false,
  },
  plugins: [
    new rspack.DefinePlugin({
      CONST_PREFIX0: JSON.stringify('prefix0'),
      CONST_SUFFIX0: JSON.stringify('suffix0'),
      CONST_PREFIX1: JSON.stringify('prefix1'),
      CONST_SUFFIX1: JSON.stringify('suffix1'),
      CONST_PREFIX2: JSON.stringify('prefix2'),
      CONST_SUFFIX2: JSON.stringify('suffix2'),
      CONST_PREFIX3: JSON.stringify('prefix3'),
      CONST_SUFFIX3: JSON.stringify('suffix3'),
      CONST_PREFIX4: JSON.stringify('prefix4'),
      CONST_SUFFIX4: JSON.stringify('suffix4'),
      DEFINED_EXPRESSION: 'foobar',
    }),
  ],
});
