/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  target: 'node',
  optimization: {
    concatenateModules: true,
    minimize: false,
  },
  module: {
    rules: [
      {
        test: /\.module\.css$/,
        type: 'css/module',
        parser: {
          namedExports: false,
        },
        generator: {
          exportsOnly: true,
          localIdentName: '[local]',
        },
      },
    ],
  },
};
