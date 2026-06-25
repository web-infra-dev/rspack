/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    './no-trusted-types.web.js': 'commonjs ./no-trusted-types.web.js',
  },
  target: 'web',
  output: {
    // TODO should be `[name].web.js`
    chunkFilename: 'no-trusted-types.web.js',
    crossOriginLoading: 'anonymous',
    trustedTypes: true,
  },
  // performance: {
  // 	hints: false
  // },
  optimization: {
    minimize: false,
  },
};
