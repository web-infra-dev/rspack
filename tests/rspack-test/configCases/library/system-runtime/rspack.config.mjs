/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  mode: 'development',
  entry: {
    main: './index.js',
  },
  output: {
    filename: '[name].js',
    library: {
      type: 'system',
    },
  },
  optimization: {
    runtimeChunk: {
      name: 'runtime',
    },
  },
};
