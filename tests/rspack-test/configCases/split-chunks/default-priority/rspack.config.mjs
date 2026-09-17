/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    filename: '[name].js',
  },
  target: 'web',
  optimization: {
    splitChunks: {
      chunks: 'all',
      cacheGroups: {
        // priority: lib-b > lib-js > lib-a
        vendor: {
          test: /\.js/,
          name: 'lib-js',
          priority: -10,
          enforce: true,
        },
        a: {
          test: /a\.js/,
          name: 'lib-a',
          priority: -30,
          enforce: true,
        },
        b: {
          test: /b\.js/,
          name: 'lib-b',
          enforce: true,
        },
      },
    },
  },
};
