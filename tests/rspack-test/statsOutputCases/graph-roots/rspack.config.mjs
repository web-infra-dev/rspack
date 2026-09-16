/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  entry: './index.js',
  optimization: {
    splitChunks: false,
  },
  stats: {
    all: false,
    chunks: true,
    chunkModules: true,
    dependentModules: false,
  },
};
