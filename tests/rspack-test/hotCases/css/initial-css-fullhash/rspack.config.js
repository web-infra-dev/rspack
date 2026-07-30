/** @type {import("@rspack/core").Configuration} */
module.exports = {
  output: {
    cssFilename: 'bundle.[fullhash].css',
    cssChunkFilename: '[name].css',
  },
  module: {
    rules: [
      {
        test: /\.css/,
        type: 'css/auto',
      },
    ],
  },
};
