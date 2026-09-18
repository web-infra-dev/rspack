/** @type {import("@rspack/core").Configuration} */
export default {
  devtool: false,
  mode: 'development',
  optimization: {
    runtimeChunk: 'single',
    splitChunks: {
      cacheGroups: {
        common: {
          chunks: 'all',
          minChunks: 1,
          name: 'common',
          enforce: true,
        },
      },
    },
  },
  stats: {
    entrypoints: true,
    assets: true,
    modules: true,
  },
};
