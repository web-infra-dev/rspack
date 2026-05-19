module.exports = {
  externals: {
    'ws-ns': 'module-import webpack-sources/lib/index.js',
    'ws-cjs': 'node-commonjs webpack-sources/lib/index.js',
  },
};
