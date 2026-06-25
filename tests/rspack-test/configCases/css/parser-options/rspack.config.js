/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  mode: 'development',
  node: {
    __dirname: false,
    __filename: false,
  },
  output: {
    cssFilename: 'bundle0.css',
  },
  module: {
    parser: {
      'css/module': {
        animation: false,
      },
    },
    generator: {
      'css/module': {
        localIdentName: '[name]_module_css-[local]',
      },
    },
    rules: [
      {
        test: /animation-name\.module\.css$/,
        type: 'css/module',
      },
      {
        test: /options\.module\.css$/,
        type: 'css/module',
        parser: {
          container: false,
          customIdents: false,
          dashedIdents: false,
          function: false,
          grid: false,
          import: false,
          url: false,
        },
      },
    ],
  },
};
