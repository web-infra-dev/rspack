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
    generator: {
      'css/auto': {
        localIdentName: '[name]_module_css-[local]',
      },
    },
    rules: [
      {
        test: /\.module\.css$/,
        type: 'css/auto',
      },
    ],
  },
};
