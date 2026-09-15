/** @type {import("@rspack/core").Configuration} */
module.exports = {
  module: {
    rules: [
      {
        test: /\.js$/,
        resource: /[\\/]entry\.js$/,
        loader: './loader',
      },
      {
        test: (resource) => /\.js$/.test(resource),
        resource: /[\\/]async-entry\.js$/,
        loader: './loader',
      },
    ],
  },
};
