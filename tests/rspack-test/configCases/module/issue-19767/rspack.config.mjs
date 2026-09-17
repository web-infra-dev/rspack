/** @type {import("@rspack/core").Configuration} */
export default () => ({
  devtool: false,
  mode: 'development',
  entry: {
    main: {
      import: './index.js',
      dependOn: 'shared',
    },
    shared: './common.js',
  },
  output: {
    module: true,
    filename: '[name].mjs',
    library: {
      type: 'module',
    },
  },
  target: ['web', 'es2020'],
  optimization: {
    minimize: false,
    runtimeChunk: false,
    concatenateModules: true,
  },
});
