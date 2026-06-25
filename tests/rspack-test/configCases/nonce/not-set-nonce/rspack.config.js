/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    './chunk-without-nonce.web.js': 'commonjs ./chunk-without-nonce.web.js',
  },
  target: 'web',
  output: {
    chunkFilename: 'chunk-without-nonce.web.js',
    crossOriginLoading: 'anonymous',
    trustedTypes: true,
  },
  optimization: {
    minimize: false,
  },
};
