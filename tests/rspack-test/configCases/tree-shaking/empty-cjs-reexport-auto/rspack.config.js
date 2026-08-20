/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  target: 'node',
  amd: {},
  optimization: {
    concatenateModules: false,
    minimize: false,
    moduleIds: 'named',
    providedExports: true,
    usedExports: true,
  },
  module: {
    noParse: /no-parse\.js$/,
    rules: [
      {
        test: /\.ts$/,
        type: 'javascript/auto',
      },
      {
        test: /disabled-commonjs\.js$/,
        parser: {
          commonjs: {
            exports: false,
          },
        },
      },
    ],
  },
  stats: {
    modules: true,
    providedExports: true,
  },
};
