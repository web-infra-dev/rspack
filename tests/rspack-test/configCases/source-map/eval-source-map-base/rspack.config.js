/** @type {import("@rspack/core").Configuration} */
module.exports = {
  node: {
    __dirname: false,
    __filename: false,
  },
  devtool: 'eval-source-map',
  externals: ['source-map'],
  externalsType: 'commonjs',
  output: {
    devtoolFallbackModuleFilenameTemplate: 'fallback://[resource-path]?[hash]',
  },
  optimization: {
    moduleIds: 'named',
  },
};
