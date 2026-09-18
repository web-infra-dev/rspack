/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: {
      import: './index',
    },
    main2: {
      import: './index2',
    },
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    runtimeChunk: 'single',
    splitChunks: {
      cacheGroups: {
        vendor: {
          chunks: 'all',
          test: /shared\.js/,
          enforce: true,
          name: 'vendor',
        },
      },
    },
  },
};
