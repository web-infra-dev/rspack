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
        test: /dynamic\.js$/,
        type: 'javascript/dynamic',
      },
      {
        test: /disabled\.js$/,
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
