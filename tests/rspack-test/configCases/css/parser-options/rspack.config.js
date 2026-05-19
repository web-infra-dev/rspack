/** @type {import("@rspack/core").Configuration} */
module.exports = {
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
      css: {
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
          customIdents: false,
          dashedIdents: false,
          import: false,
          url: false,
        },
      },
    ],
  },
};
