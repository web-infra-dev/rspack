/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  output: {
    chunkFilename: 'js/chunks/c.js',
  },
  devtool: 'source-map',
  externals: ['source-map'],
  externalsType: 'commonjs',
};
