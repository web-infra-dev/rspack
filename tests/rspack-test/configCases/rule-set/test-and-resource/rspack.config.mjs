/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /\.js$/,
        resource: /[\\/]entry\.js$/,
        loader: './loader.mjs',
      },
      {
        test: (resource) => /\.js$/.test(resource),
        resource: /[\\/]async-entry\.js$/,
        loader: './loader.mjs',
      },
    ],
  },
};
