/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    './no-trusted-types-policy-name.web.js':
      'commonjs ./no-trusted-types-policy-name.web.js',
  },
  target: 'web',
  output: {
    // TODO should be `[name].web.js`
    chunkFilename: 'no-trusted-types-policy-name.web.js',
    crossOriginLoading: 'anonymous',
  },
  // performance: {
  // 	hints: false
  // },
  optimization: {
    minimize: false,
  },
};
