/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: "./index.js",
  module: {
    rules: [
      {
        resource: /.png/,
        type: 'asset/resource',
        generator: {
          filename: '[path][name][ext]'
        }
      },
    ]
  },
};
