/**
 * @type {import('@rspack/core').Configuration}
 */
export default {
  externals: {
    fs: 'node-commonjs fs',
  },
  entry: './index.js',
  node: {
    __dirname: false,
    __filename: false,
  },
  optimization: {
    sideEffects: false,
    concatenateModules: false,
    innerGraph: false,
  },
};
