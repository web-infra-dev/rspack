/** @type {import("@rspack/core").Configuration} */
export default {
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
