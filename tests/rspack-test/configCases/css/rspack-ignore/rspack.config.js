/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset/resource',
        generator: { filename: 'resolved-[name][ext]' },
      },
      {
        test: /\.css$/,
        type: 'css/auto',
        parser: { pure: true },
      },
    ],
  },
};
