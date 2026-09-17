/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  target: 'node',
  entry: {
    main: './src/index.js',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    splitChunks: {
      chunks: 'all',
      minSize: {
        js: 1000,
        css: 1000,
      },
    },
  },
};
