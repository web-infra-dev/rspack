/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    './default-policy-name.web.js': 'commonjs ./default-policy-name.web.js',
  },
  target: 'web',
  output: {
    // TODO should be `[name].web.js`
    chunkFilename: 'default-policy-name.web.js',
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
