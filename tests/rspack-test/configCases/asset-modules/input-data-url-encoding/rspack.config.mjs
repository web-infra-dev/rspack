/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  devtool: false,
  module: {
    rules: [
      {
        dependency: 'url',
        type: 'asset',
        generator: {
          dataUrl: {
            encoding: false,
          },
        },
      },
    ],
  },
  target: 'web',
};
