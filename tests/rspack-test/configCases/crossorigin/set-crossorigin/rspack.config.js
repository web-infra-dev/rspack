/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    './crossorigin-default.web.js': 'commonjs ./crossorigin-default.web.js',
    './crossorigin-different-origin.web.js':
      'commonjs ./crossorigin-different-origin.web.js',
    './crossorigin-relative.web.js': 'commonjs ./crossorigin-relative.web.js',
    './crossorigin-same-origin.web.js':
      'commonjs ./crossorigin-same-origin.web.js',
    './crossorigin-server-relative.web.js':
      'commonjs ./crossorigin-server-relative.web.js',
  },
  target: 'web',
  output: {
    chunkFilename: '[name].web.js',
    crossOriginLoading: 'anonymous',
  },
  performance: {
    hints: false,
  },
  optimization: {
    minimize: false,
  },
};
