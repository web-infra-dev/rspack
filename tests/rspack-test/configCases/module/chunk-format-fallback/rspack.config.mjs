/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: {
      import: './index.js',
      library: { type: 'module' },
    },
  },
  output: {
    module: true,
    filename: '[name].mjs',
  },
  optimization: {
    runtimeChunk: 'single',
  },
  mode: 'development',
  devtool: false,
};
