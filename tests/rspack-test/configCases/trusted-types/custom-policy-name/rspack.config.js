/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    './trusted-types.web.js': 'commonjs ./trusted-types.web.js',
  },
  target: 'web',
  output: {
    // TODO should be `[name].web.js`
    chunkFilename: 'trusted-types.web.js',
    crossOriginLoading: 'anonymous',
    trustedTypes: 'customPolicyName',
  },
  // performance: {
  // 	hints: false
  // },
  optimization: {
    minimize: false,
  },
};
