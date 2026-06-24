/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: {
    async: './async.js',
    other: './other.js',
  },
  output: {
    filename: '[name].js',
    chunkLoading: 'async-node',
    library: {
      name: 'MyLib',
      type: 'commonjs-module',
    },
  },
  optimization: {
    splitChunks: {
      minSize: 0,
      cacheGroups: {
        lib1: {
          test: /lib-1/,
          name: 'lib1',
          chunks: 'all',
          priority: 3,
        },
      },
    },
    // Avoid the default export of lib-*.js is inlined, which causes the splitChunks lib* cache group disappear.
    inlineExports: false,
  },
  target: 'node',
};
